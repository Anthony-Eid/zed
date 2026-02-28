use std::process::{Child, Command, Stdio};
use std::sync::Arc;

use anyhow::Context as _;
use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    SharedString, Styled, Task, Window, actions, div, prelude::*, px, rgb, webview,
};
use ui::prelude::*;
use workspace::Workspace;
use workspace::item::Item;

actions!(cad, [OpenCadViewer]);

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
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(webview(url).size_full().rounded_md())
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

    fn show_toolbar(&self) -> bool {
        false
    }
}

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        CadViewer::register(workspace, window, cx);
    })
    .detach();
}
