#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Point, Render,
    Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size, webview,
};
use gpui_platform::application;

const MIN_WEBVIEW_WIDTH: f32 = 260.0;
const MAX_WEBVIEW_WIDTH: f32 = 900.0;
const INITIAL_WEBVIEW_WIDTH: f32 = 420.0;
const SPLITTER_WIDTH: f32 = 6.0;

struct WebviewMultiEmbedExample {
    show_webview: bool,
    webview_width: Pixels,
    resizing_webview: bool,
    resize_origin_x: Pixels,
    resize_origin_width: Pixels,
}

impl WebviewMultiEmbedExample {
    fn start_resize(&mut self, mouse_position: Point<Pixels>) {
        self.resizing_webview = true;
        self.resize_origin_x = mouse_position.x;
        self.resize_origin_width = self.webview_width;
    }

    fn stop_resize(&mut self) {
        self.resizing_webview = false;
    }

    fn update_resize(&mut self, mouse_position: Point<Pixels>, cx: &mut Context<Self>) {
        if !self.resizing_webview {
            return;
        }

        let delta = self.resize_origin_x - mouse_position.x;
        let target_width = self.resize_origin_width + delta;
        let clamped_width = target_width.clamp(px(MIN_WEBVIEW_WIDTH), px(MAX_WEBVIEW_WIDTH));

        if clamped_width != self.webview_width {
            self.webview_width = clamped_width;
            cx.notify();
        }
    }
}

impl Render for WebviewMultiEmbedExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let toggle_label = if self.show_webview {
            "Turn webview off"
        } else {
            "Turn webview on"
        };

        let show_webview = self.show_webview;
        let webview_width = self.webview_width;

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x0f172a))
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .bg(rgb(0x111827))
                    .text_color(rgb(0xffffff))
                    .child("Single-webview embed example")
                    .child(
                        div()
                            .id("toggle-webview")
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x1d4ed8))
                            .hover(|style| style.bg(rgb(0x1e40af)))
                            .active(|style| style.opacity(0.85))
                            .text_color(rgb(0xffffff))
                            .cursor_pointer()
                            .child(toggle_label)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_webview = !this.show_webview;
                                this.stop_resize();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .p_2()
                    .bg(rgb(0x020617))
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                        this.update_resize(event.position, cx);
                    }))
                    .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.stop_resize();
                    }))
                    .child(
                        div()
                            .id("content-pane")
                            .flex_1()
                            .h_full()
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(0x334155))
                            .bg(rgb(0x0b1220))
                            .text_color(rgb(0x94a3b8))
                            .text_sm()
                            .p_3()
                            .child(
                                "Main content area. Drag the splitter to resize the webview pane.",
                            ),
                    )
                    .when(show_webview, |container| {
                        container.child(
                            div()
                                .id("webview-splitter")
                                .w(px(SPLITTER_WIDTH))
                                .mx_1()
                                .h_full()
                                .cursor_col_resize()
                                .bg(rgb(0x1e293b))
                                .hover(|style| style.bg(rgb(0x334155)))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                                        this.start_resize(event.position);
                                    }),
                                ),
                        )
                    })
                    .child(
                        div()
                            .id("webview-pane")
                            .when(show_webview, |pane| {
                                pane.w(webview_width)
                                    .min_w(px(MIN_WEBVIEW_WIDTH))
                                    .max_w(px(MAX_WEBVIEW_WIDTH))
                            })
                            .when(!show_webview, |pane| pane.w(px(0.0)).overflow_hidden())
                            .h_full()
                            .rounded_md()
                            .overflow_hidden()
                            .when(show_webview, |pane| {
                                pane.border_1().border_color(rgb(0x334155))
                            })
                            .child(
                                webview("https://example.com")
                                    .size_full()
                                    .rounded_md()
                                    .visible(show_webview),
                            ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x111827))
                    .text_color(rgb(0x94a3b8))
                    .text_sm()
                    .child(
                        "Use the button to create/destroy the single webview. Drag the splitter to resize it.",
                    ),
            )
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.0), px(820.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| WebviewMultiEmbedExample {
                    show_webview: true,
                    webview_width: px(INITIAL_WEBVIEW_WIDTH),
                    resizing_webview: false,
                    resize_origin_x: px(0.0),
                    resize_origin_width: px(INITIAL_WEBVIEW_WIDTH),
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}
