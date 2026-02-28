#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, Pixels, SharedString, Size, Window, WindowBounds, WindowOptions, div,
    point, prelude::*, px, rgb, size,
};
use gpui_platform::application;

struct WebviewSplitExample {
    webview_url: SharedString,
    did_set_initial_url: bool,
    last_webview_bounds: Option<Bounds<Pixels>>,
}

impl WebviewSplitExample {
    fn right_half_bounds(window_size: Size<Pixels>) -> Bounds<Pixels> {
        let half_width = window_size.width * 0.5;
        Bounds::new(
            point(half_width, px(0.0)),
            size(half_width, window_size.height),
        )
    }

    fn ensure_webview(&mut self, window: &mut Window) {
        let right_half_bounds = Self::right_half_bounds(window.viewport_size());

        let should_update_bounds = self
            .last_webview_bounds
            .is_none_or(|last_bounds| last_bounds != right_half_bounds);

        if should_update_bounds {
            window.set_webview_bounds(Some(right_half_bounds));
            self.last_webview_bounds = Some(right_half_bounds);
        }

        if !self.did_set_initial_url {
            window.load_webview_url(self.webview_url.as_ref());
            self.did_set_initial_url = true;
        }
    }
}

impl Render for WebviewSplitExample {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_webview(window);

        div()
            .size_full()
            .flex()
            .child(
                div()
                    .w_1_2()
                    .h_full()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .items_center()
                    .gap_2()
                    .bg(rgb(0x1f2937))
                    .text_color(rgb(0xffffff))
                    .child("GPUI pane (left)")
                    .child("WKWebView pane (right)")
                    .child("Resize the window to verify alignment"),
            )
            .child(div().w_1_2().h_full())
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| WebviewSplitExample {
                    webview_url: "https://example.com".into(),
                    did_set_initial_url: false,
                    last_webview_bounds: None,
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
