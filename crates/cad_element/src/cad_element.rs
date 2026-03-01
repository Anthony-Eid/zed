use std::io::Write as _;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use agent::AgentTool;
use agent_client_protocol::ToolKind;
use anyhow::Context as _;
use base64::Engine as _;
use gpui::{
    App, AppContext as _, Context, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, SharedString, Styled, Task, Window, actions, div, prelude::*, px, rgb,
    webview,
};
use language_model::{LanguageModelImage, LanguageModelToolResultContent};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ui::prelude::*;
use workspace::Workspace;
use workspace::item::Item;

actions!(cad, [OpenCadViewer]);

/// Executes a build123d/CadQuery Python script and sends the resulting 3D model to the running
/// ocp_vscode viewer so it can be inspected interactively.
///
/// The script must use `show(...)` or `show_object(...)` from `ocp_vscode` to display objects.
/// The viewer must already be open and running (use the `open_cad_viewer` action first).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RenderCadCodeInput {
    /// The complete Python script to execute. Must import build123d or CadQuery and call
    /// show(...) or show_object(...) to send the model to the viewer.
    pub code: String,
    /// The port the ocp_vscode viewer is listening on. Defaults to 3939 if not specified.
    pub port: Option<u16>,
}

pub struct RenderCadCodeTool;

impl AgentTool for RenderCadCodeTool {
    type Input = RenderCadCodeInput;
    type Output = String;

    const NAME: &'static str = "render_cad_code";

    fn kind() -> ToolKind {
        ToolKind::Execute
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        match input {
            Ok(input) => {
                let first_line = input.code.lines().next().unwrap_or("").trim();
                if first_line.is_empty() {
                    "Render CAD code".into()
                } else {
                    format!("Render: {}", first_line).into()
                }
            }
            Err(_) => "Render CAD code".into(),
        }
    }

    fn run(
        self: Arc<Self>,
        input: agent::ToolInput<Self::Input>,
        _event_stream: agent::ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        cx.background_spawn(async move {
            let input = input
                .recv()
                .await
                .map_err(|e| format!("Failed to receive tool input: {e}"))?;

            let port = input.port.unwrap_or(DEFAULT_PORT);

            let mut temp_file = tempfile::Builder::new()
                .suffix(".py")
                .tempfile()
                .map_err(|e| format!("Failed to create temp file: {e}"))?;

            let script = format!(
                "import ocp_vscode\nocp_vscode.set_port({})\n{}",
                port, input.code
            );

            temp_file
                .write_all(script.as_bytes())
                .map_err(|e| format!("Failed to write script: {e}"))?;

            let script_path = temp_file.path().to_string_lossy().into_owned();

            let output = Command::new("uv")
                .args([
                    "run",
                    "--with",
                    "ocp-vscode",
                    "--with",
                    "build123d",
                    "--",
                    "python",
                    &script_path,
                ])
                .output()
                .or_else(|_| Command::new("python3").args(["--", &script_path]).output())
                .map_err(|e| format!("Failed to run Python: {e}"))?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim().is_empty() {
                    Ok("CAD model rendered successfully.".to_string())
                } else {
                    Ok(format!(
                        "CAD model rendered successfully.\n{}",
                        stdout.trim()
                    ))
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut message = String::from("Script execution failed.");
                if !stderr.trim().is_empty() {
                    message.push('\n');
                    message.push_str(stderr.trim());
                }
                if !stdout.trim().is_empty() {
                    message.push('\n');
                    message.push_str(stdout.trim());
                }
                Err(message)
            }
        })
    }
}

/// Takes a screenshot of the current 3D model displayed in the ocp_vscode CAD viewer and returns
/// it as an image so the agent can visually verify the rendered output.
///
/// The viewer must already be open and displaying a model. Use `render_cad_code` first to send
/// a model to the viewer.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ScreenshotCadViewerInput {
    /// The port the ocp_vscode viewer is listening on. Defaults to 3939 if not specified.
    pub port: Option<u16>,
}

pub struct ScreenshotCadViewerTool;

impl AgentTool for ScreenshotCadViewerTool {
    type Input = ScreenshotCadViewerInput;
    type Output = LanguageModelToolResultContent;

    const NAME: &'static str = "screenshot_cad_viewer";

    fn kind() -> ToolKind {
        ToolKind::Read
    }

    fn initial_title(
        &self,
        _input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        "Screenshot CAD viewer".into()
    }

    fn run(
        self: Arc<Self>,
        input: agent::ToolInput<Self::Input>,
        _event_stream: agent::ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        cx.background_spawn(async move {
            take_screenshot(input).await.map_err(|e| {
                LanguageModelToolResultContent::Text(Arc::from(e.to_string().as_str()))
            })
        })
    }
}

async fn take_screenshot(
    input: agent::ToolInput<ScreenshotCadViewerInput>,
) -> anyhow::Result<LanguageModelToolResultContent> {
    let input = input
        .recv()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to receive tool input: {e}"))?;

    let port = input.port.unwrap_or(DEFAULT_PORT);

    let temp_path = std::env::temp_dir().join("cad_screenshot.png");
    let temp_path_str = temp_path.to_string_lossy().into_owned();

    // Ask the viewer to save a screenshot to the temp path by running a small Python helper.
    let script = format!(
        "import ocp_vscode\n\
         ocp_vscode.set_port({port})\n\
         ocp_vscode.save_screenshot('{temp_path_str}', port={port})\n"
    );

    let mut temp_script = tempfile::Builder::new()
        .suffix(".py")
        .tempfile()
        .context("Failed to create temp script")?;

    temp_script
        .write_all(script.as_bytes())
        .context("Failed to write screenshot script")?;

    let script_path = temp_script.path().to_string_lossy().into_owned();

    let output = Command::new("uv")
        .args([
            "run",
            "--with",
            "ocp-vscode",
            "--with",
            "build123d",
            "--",
            "python",
            &script_path,
        ])
        .output()
        .or_else(|_| Command::new("python3").args(["--", &script_path]).output())
        .context("Failed to run screenshot command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Screenshot command failed: {}", stderr.trim());
    }

    // Poll briefly for the file to appear (save_screenshot polls internally but we
    // double-check here to handle any timing edge cases).
    let mut png_bytes = None;
    for _ in 0..20 {
        if temp_path.exists() {
            match std::fs::read(&temp_path) {
                Ok(bytes) if !bytes.is_empty() => {
                    png_bytes = Some(bytes);
                    break;
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    let bytes = png_bytes.context(
        "Screenshot file was not created within the timeout. \
         Is the CAD viewer open and showing a model?",
    )?;

    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let image = LanguageModelImage {
        source: SharedString::from(encoded),
        size: None,
    };

    Ok(LanguageModelToolResultContent::Image(image))
}

const DEFAULT_PORT: u16 = 3939;
const DEFAULT_HOST: &str = "127.0.0.1";

struct OcpServer {
    process: Child,
    port: u16,
}

impl OcpServer {
    fn spawn(port: u16) -> anyhow::Result<Self> {
        let port_str = port.to_string();

        // Prefer `uv run` which creates an ephemeral env with the right packages
        // automatically — no manual venv setup needed.
        let process = Command::new("uv")
            .args([
                "run",
                "--with",
                "ocp-vscode",
                "--with",
                "build123d",
                "--",
                "python",
                "-m",
                "ocp_vscode",
                "--port",
                &port_str,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .or_else(|_| {
                // Fall back to a direct python3 invocation if uv isn't available.
                Command::new("python3")
                    .args(["-m", "ocp_vscode", "--port", &port_str])
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
            })
            .context(
                "Failed to start ocp_vscode server. \
                 Install `uv` (https://docs.astral.sh/uv/) or run \
                 `pip install ocp-vscode build123d` manually.",
            )?;

        Ok(Self { process, port })
    }

    fn url(&self) -> String {
        format!("http://{}:{}", DEFAULT_HOST, self.port)
    }
}

impl Drop for OcpServer {
    fn drop(&mut self) {
        if let Err(error) = self.process.kill() {
            log::warn!("Failed to kill ocp_vscode server process: {error}");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServerStatus {
    Starting,
    Running,
    Failed,
}

pub struct CadViewer {
    focus_handle: FocusHandle,
    server: Option<Arc<OcpServer>>,
    server_url: Option<String>,
    status: ServerStatus,
    error_message: Option<SharedString>,
    port: u16,
    active: bool,
    _start_task: Task<()>,
}

impl CadViewer {
    pub fn new(port: u16, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let start_task = cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move { OcpServer::spawn(port) })
                .await;

            // Give the server a moment to bind the port before the webview tries to connect.
            cx.background_executor()
                .timer(std::time::Duration::from_secs(2))
                .await;

            this.update(cx, |viewer, cx| match result {
                Ok(server) => {
                    let url = server.url();
                    viewer.server_url = Some(url);
                    viewer.server = Some(Arc::new(server));
                    viewer.status = ServerStatus::Running;
                    cx.notify();
                }
                Err(error) => {
                    log::error!("Failed to start CAD visualization server: {error:#}");
                    viewer.status = ServerStatus::Failed;
                    viewer.error_message = Some(SharedString::from(format!("{error:#}")));
                    cx.notify();
                }
            })
            .ok();
        });

        Self {
            focus_handle: cx.focus_handle(),
            server: None,
            server_url: None,
            status: ServerStatus::Starting,
            error_message: None,
            port,
            active: true,
            _start_task: start_task,
        }
    }

    fn render_loading(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .bg(cx.theme().colors().editor_background)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_color(cx.theme().colors().text)
                            .text_lg()
                            .child("Starting CAD visualization server..."),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().colors().text_muted)
                            .text_sm()
                            .child(format!("Launching ocp_vscode on port {}", self.port)),
                    ),
            )
    }

    fn render_error(&self, cx: &Context<Self>) -> impl IntoElement {
        let message = self
            .error_message
            .clone()
            .unwrap_or_else(|| "Unknown error".into());

        div()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .bg(cx.theme().colors().editor_background)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .max_w(px(500.0))
                    .child(
                        div()
                            .text_color(rgb(0xef4444))
                            .text_lg()
                            .child("Failed to start CAD server"),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().colors().text_muted)
                            .text_sm()
                            .child(message),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().colors().text_muted)
                            .text_xs()
                            .child("Install uv (https://docs.astral.sh/uv/) or run: pip install ocp-vscode build123d"),
                    ),
            )
    }

    fn render_viewer(&self, cx: &Context<Self>) -> impl IntoElement {
        let url = self
            .server_url
            .clone()
            .unwrap_or_else(|| format!("http://{}:{}", DEFAULT_HOST, self.port));

        div()
            .id("cad-element-viewer")
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(
                webview(url)
                    .id("cad-webview")
                    .size_full()
                    .rounded_md()
                    .visible(self.active),
            )
    }

    pub fn register(workspace: &mut Workspace, _window: &mut Window, _cx: &mut Context<Workspace>) {
        workspace.register_action(move |workspace, _: &OpenCadViewer, window, cx| {
            let view = cx.new(|cx| CadViewer::new(DEFAULT_PORT, window, cx));
            workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
        });
    }
}

impl Render for CadViewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.active = true;

        let content = match self.status {
            ServerStatus::Starting => self.render_loading(cx).into_any_element(),
            ServerStatus::Failed => self.render_error(cx).into_any_element(),
            ServerStatus::Running => self.render_viewer(cx).into_any_element(),
        };

        div()
            .id("cad-viewer")
            .key_context("CadViewer")
            .track_focus(&self.focus_handle(cx))
            .size_full()
            .child(content)
    }
}

impl Focusable for CadViewer {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<()> for CadViewer {}

impl Item for CadViewer {
    type Event = ();

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::from("CAD Viewer")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Eye))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::from(format!(
            "build123d CAD Viewer (port {})",
            self.port
        )))
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("cad viewer: open")
    }

    fn to_item_events(_event: &Self::Event, _f: &mut dyn FnMut(workspace::item::ItemEvent)) {}

    fn deactivated(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.active = false;
        cx.notify();
    }

    fn workspace_deactivated(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.active = false;
        cx.notify();
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}

pub fn register_tools(thread: &mut agent::Thread) {
    thread.add_tool(RenderCadCodeTool);
    thread.add_tool(ScreenshotCadViewerTool);
}

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        CadViewer::register(workspace, window, cx);
    })
    .detach();

    cx.observe_new(|thread: &mut agent::Thread, _window, _cx| {
        register_tools(thread);
    })
    .detach();
}
