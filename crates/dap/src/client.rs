use crate::transport::{self, Events, Payload, Request, Transport};
use anyhow::{anyhow, Context, Result};

use dap_types::{
    events::Process,
    requests::{
        Attach, ConfigurationDone, Continue, Initialize, Launch, Next, Pause, SetBreakpoints,
        StepBack, StepIn, StepOut,
    },
    AttachRequestArguments, ConfigurationDoneArguments, ContinueArguments,
    InitializeRequestArgumentsPathFormat, LaunchRequestArguments, NextArguments, PauseArguments,
    Scope, SetBreakpointsArguments, SetBreakpointsResponse, Source, SourceBreakpoint, StackFrame,
    StepBackArguments, StepInArguments, StepOutArguments, SteppingGranularity, Variable,
};
use futures::{
    channel::mpsc::{channel, unbounded, UnboundedReceiver, UnboundedSender},
    AsyncBufRead, AsyncReadExt, AsyncWrite, SinkExt as _, StreamExt,
};
use gpui::{AppContext, AsyncAppContext};
use parking_lot::{Mutex, MutexGuard};
use serde_json::Value;
use smol::{
    io::BufReader,
    net::TcpStream,
    process::{self, Child},
};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
    process::Stdio,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use task::{DebugAdapterConfig, DebugConnectionType, DebugRequestType};
use util::ResultExt;

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ThreadStatus {
    #[default]
    Running,
    Stopped,
    Ended,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DebugAdapterClientId(pub usize);

#[derive(Debug, Default, Clone)]
pub struct ThreadState {
    pub status: ThreadStatus,
    pub stack_frames: Vec<StackFrame>,
    pub scopes: HashMap<u64, Vec<Scope>>, // stack_frame_id -> scopes
    pub variables: HashMap<u64, Vec<Variable>>, // scope.variable_reference -> variables
    pub current_stack_frame_id: Option<u64>,
}

pub struct DebugAdapterClient {
    id: DebugAdapterClientId,
    _process: Option<Child>,
    server_tx: UnboundedSender<Payload>,
    request_count: AtomicU64,
    capabilities: Option<dap_types::Capabilities>,
    config: DebugAdapterConfig,
    client_rx: Arc<smol::lock::Mutex<UnboundedReceiver<Payload>>>,
    thread_states: Arc<Mutex<HashMap<u64, ThreadState>>>, // thread_id -> thread_state
}

impl DebugAdapterClient {
    pub async fn new(
        id: DebugAdapterClientId,
        config: DebugAdapterConfig,
        command: &str,
        args: Vec<&str>,
        project_path: PathBuf,
        cx: &mut AsyncAppContext,
    ) -> Result<Self> {
        match config.connection {
            DebugConnectionType::TCP => {
                Self::create_tcp_client(id, config, command, args, project_path, cx).await
            }
            DebugConnectionType::STDIO => {
                Self::create_stdio_client(id, config, command, args, project_path, cx).await
            }
        }
    }

    async fn create_tcp_client(
        id: DebugAdapterClientId,
        config: DebugAdapterConfig,
        command: &str,
        args: Vec<&str>,
        project_path: PathBuf,
        cx: &mut AsyncAppContext,
    ) -> Result<Self> {
        let mut command = process::Command::new(command);
        command
            .current_dir(project_path)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let process = command
            .spawn()
            .with_context(|| "failed to spawn command.")?;

        // give the adapter some time to spin up the tcp server
        cx.background_executor()
            .timer(Duration::from_millis(1000))
            .await;

        let address = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), config.port);

        let (rx, tx) = TcpStream::connect(address).await?.split();

        Self::handle_transport(
            id,
            config,
            Box::new(BufReader::new(rx)),
            Box::new(tx),
            None,
            Some(process),
            cx,
        )
    }

    async fn create_stdio_client(
        id: DebugAdapterClientId,
        config: DebugAdapterConfig,
        command: &str,
        args: Vec<&str>,
        project_path: PathBuf,
        cx: &mut AsyncAppContext,
    ) -> Result<Self> {
        let mut command = process::Command::new(command);
        command
            .current_dir(project_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut process = command
            .spawn()
            .with_context(|| "failed to spawn command.")?;

        let stdin = Box::new(process.stdin.take().unwrap());
        let stdout = Box::new(BufReader::new(process.stdout.take().unwrap()));
        let stderr = Box::new(BufReader::new(process.stderr.take().unwrap()));

        Self::handle_transport(id, config, stdout, stdin, Some(stderr), Some(process), cx)
    }

    pub fn handle_transport(
        id: DebugAdapterClientId,
        config: DebugAdapterConfig,
        rx: Box<dyn AsyncBufRead + Unpin + Send>,
        tx: Box<dyn AsyncWrite + Unpin + Send>,
        err: Option<Box<dyn AsyncBufRead + Unpin + Send>>,
        process: Option<Child>,
        cx: &mut AsyncAppContext,
    ) -> Result<Self> {
        let (server_rx, server_tx) = Transport::start(rx, tx, err, cx);
        let (client_tx, client_rx) = unbounded::<Payload>();

        let client_rx = Arc::new(smol::lock::Mutex::new(client_rx));

        let client = Self {
            id,
            config,
            client_rx,
            _process: process,
            capabilities: None,
            server_tx: server_tx.clone(),
            request_count: AtomicU64::new(0),
            thread_states: Arc::new(Mutex::new(HashMap::new())),
        };

        cx.spawn(move |_| Self::handle_recv(server_rx, server_tx, client_tx))
            .detach();

        Ok(client)
    }

    pub async fn handle_events<F>(
        client: Arc<Self>,
        mut event_handler: F,
        cx: AsyncAppContext,
    ) -> Result<()>
    where
        F: FnMut(Events, &mut AppContext) + 'static + Send + Sync + Clone,
    {
        while let Some(payload) = client.client_rx.lock().await.next().await {
            cx.update(|cx| match payload {
                Payload::Event(event) => event_handler(*event, cx),
                _ => unreachable!(),
            })?;
        }

        anyhow::Ok(())
    }

    async fn handle_recv(
        mut server_rx: UnboundedReceiver<Payload>,
        mut server_tx: UnboundedSender<Payload>,
        mut client_tx: UnboundedSender<Payload>,
    ) {
        while let Some(payload) = server_rx.next().await {
            match payload {
                Payload::Event(ev) => client_tx.send(Payload::Event(ev)).await.log_err(),
                Payload::Response(res) => server_tx.send(Payload::Response(res)).await.log_err(),
                Payload::Request(req) => client_tx.send(Payload::Request(req)).await.log_err(),
            };
        }
    }

    pub async fn request<R: dap_types::requests::Request>(
        &self,
        arguments: R::Arguments,
    ) -> Result<R::Response> {
        let serialized_arguments = serde_json::to_value(arguments)?;

        let (callback_tx, mut callback_rx) = channel::<Result<transport::Response>>(1);

        let request = Request {
            back_ch: Some(callback_tx),
            seq: self.next_request_id(),
            command: R::COMMAND.to_string(),
            arguments: Some(serialized_arguments),
        };

        self.server_tx
            .clone()
            .send(Payload::Request(request))
            .await?;

        let response = callback_rx.next().await.ok_or(anyhow!("no response"))??;

        match response.success {
            true => Ok(serde_json::from_value(response.body.unwrap_or_default())?),
            false => Err(anyhow!("Request failed")),
        }
    }

    pub fn id(&self) -> DebugAdapterClientId {
        self.id
    }

    pub fn config(&self) -> DebugAdapterConfig {
        self.config.clone()
    }

    pub fn request_type(&self) -> DebugRequestType {
        self.config.request.clone()
    }

    pub fn next_request_id(&self) -> u64 {
        self.request_count.fetch_add(1, Ordering::Relaxed)
    }

    pub fn update_thread_state_status(&self, thread_id: u64, status: ThreadStatus) {
        if let Some(thread_state) = self.thread_states().get_mut(&thread_id) {
            thread_state.status = status;
        };
    }

    pub fn thread_states(&self) -> MutexGuard<HashMap<u64, ThreadState>> {
        self.thread_states.lock()
    }

    pub fn thread_state_by_id(&self, thread_id: u64) -> ThreadState {
        self.thread_states.lock().get(&thread_id).cloned().unwrap()
    }

    pub async fn initialize(&mut self) -> Result<dap_types::Capabilities> {
        let args = dap_types::InitializeRequestArguments {
            client_id: Some("zed".to_owned()),
            client_name: Some("Zed".to_owned()),
            adapter_id: self.config.id.clone(),
            locale: Some("en-us".to_owned()),
            path_format: Some(InitializeRequestArgumentsPathFormat::Path),
            supports_variable_type: Some(true),
            supports_variable_paging: Some(false),
            supports_run_in_terminal_request: Some(false), // TODO: we should support this
            supports_memory_references: Some(true),
            supports_progress_reporting: Some(true),
            supports_invalidated_event: Some(false),
            lines_start_at1: Some(true),
            columns_start_at1: Some(true),
            supports_memory_event: Some(true),
            supports_args_can_be_interpreted_by_shell: None,
            supports_start_debugging_request: Some(true),
        };

        let capabilities = self.request::<Initialize>(args).await?;

        self.capabilities = Some(capabilities.clone());

        Ok(capabilities)
    }

    pub async fn launch(&self, args: Option<Value>) -> Result<()> {
        self.request::<Launch>(LaunchRequestArguments {
            raw: args.unwrap_or(Value::Null),
        })
        .await
    }

    pub async fn attach(&self, args: Option<Value>) -> Result<()> {
        self.request::<Attach>(AttachRequestArguments {
            raw: args.unwrap_or(Value::Null),
        })
        .await
    }

    pub async fn resume(&self, thread_id: u64) {
        self.request::<Continue>(ContinueArguments {
            thread_id,
            single_thread: Some(true),
        })
        .await
        .log_err();
    }

    pub async fn step_over(&self, thread_id: u64) {
        self.request::<Next>(NextArguments {
            thread_id,
            granularity: Some(SteppingGranularity::Statement),
            single_thread: Some(true),
        })
        .await
        .log_err();
    }

    pub async fn step_in(&self, thread_id: u64) {
        self.request::<StepIn>(StepInArguments {
            thread_id,
            target_id: None,
            granularity: Some(SteppingGranularity::Statement),
            single_thread: Some(true),
        })
        .await
        .log_err();
    }

    pub async fn step_out(&self, thread_id: u64) {
        self.request::<StepOut>(StepOutArguments {
            thread_id,
            granularity: Some(SteppingGranularity::Statement),
            single_thread: Some(true),
        })
        .await
        .log_err();
    }

    pub async fn step_back(&self, thread_id: u64) {
        self.request::<StepBack>(StepBackArguments {
            thread_id,
            single_thread: Some(true),
            granularity: Some(SteppingGranularity::Statement),
        })
        .await
        .log_err();
    }

    pub async fn restart(&self, thread_id: u64) {
        self.request::<StepBack>(StepBackArguments {
            thread_id,
            single_thread: Some(true),
            granularity: Some(SteppingGranularity::Statement),
        })
        .await
        .log_err();
    }

    pub async fn pause(&self, thread_id: u64) {
        self.request::<Pause>(PauseArguments { thread_id })
            .await
            .log_err();
    }

    pub async fn set_breakpoints(
        &self,
        path: PathBuf,
        breakpoints: Option<Vec<SourceBreakpoint>>,
    ) -> Result<SetBreakpointsResponse> {
        let adapter_data = self.config.request_args.clone().map(|c| c.args);

        self.request::<SetBreakpoints>(SetBreakpointsArguments {
            source: Source {
                path: Some(String::from(path.to_string_lossy())),
                name: None,
                source_reference: None,
                presentation_hint: None,
                origin: None,
                sources: None,
                adapter_data,
                checksums: None,
            },
            breakpoints,
            source_modified: None,
            lines: None,
        })
        .await
    }

    pub async fn configuration_done(&self) -> Result<()> {
        self.request::<ConfigurationDone>(ConfigurationDoneArguments)
            .await
    }
}
