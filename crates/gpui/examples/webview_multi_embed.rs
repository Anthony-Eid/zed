#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
    webview,
};
use gpui_platform::application;

struct WebviewMultiEmbedExample {
    show_second_webview: bool,
}

impl Render for WebviewMultiEmbedExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let toggle_label = if self.show_second_webview {
            "Remove second webview"
        } else {
            "Add second webview"
        };

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
                    .child("Multi-webview embed example")
                    .child(
                        div()
                            .id("toggle-second-webview")
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
                                this.show_second_webview = !this.show_second_webview;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .gap_2()
                    .p_2()
                    .bg(rgb(0x020617))
                    .child(
                        div()
                            .id("webview-left-pane")
                            .flex_1()
                            .h_full()
                            .rounded_md()
                            .overflow_hidden()
                            .border_1()
                            .border_color(rgb(0x334155))
                            .child(webview("https://example.com").size_full()),
                    )
                    .when(self.show_second_webview, |container| {
                        container.child(
                            div()
                                .id("webview-right-pane")
                                .flex_1()
                                .h_full()
                                .rounded_md()
                                .overflow_hidden()
                                .border_1()
                                .border_color(rgb(0x334155))
                                .child(webview("https://www.rust-lang.org").size_full()),
                        )
                    }),
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
                        "Resize the window and toggle the second pane to verify create/update/destroy lifecycle.",
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
                    show_second_webview: true,
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
