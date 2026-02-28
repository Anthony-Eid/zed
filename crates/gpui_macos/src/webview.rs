use cocoa::{
    appkit::{NSViewHeightSizable, NSViewWidthSizable},
    base::{id, nil},
    foundation::{NSRect, NSString},
};
use objc::{class, msg_send, sel, sel_impl};

#[link(name = "WebKit", kind = "framework")]
unsafe extern "C" {}

/// Creates a `WKWebView` configured to resize with its parent view.
#[allow(dead_code)]
pub(crate) unsafe fn create_wkwebview(frame: NSRect) -> Option<id> {
    let configuration: id = msg_send![class!(WKWebViewConfiguration), new];
    if configuration == nil {
        return None;
    }

    let webview_alloc: id = msg_send![class!(WKWebView), alloc];
    if webview_alloc == nil {
        let _: () = msg_send![configuration, release];
        return None;
    }

    let webview: id = msg_send![webview_alloc, initWithFrame: frame configuration: configuration];
    let _: () = msg_send![configuration, release];

    if webview == nil {
        return None;
    }

    let _: () = msg_send![webview, setAutoresizingMask: NSViewWidthSizable | NSViewHeightSizable];
    Some(webview)
}

/// Loads a URL into a `WKWebView`.
#[allow(dead_code)]
pub(crate) unsafe fn load_url(webview: id, url: &str) -> bool {
    if webview == nil {
        return false;
    }

    let trimmed_url = url.trim();
    if trimmed_url.is_empty() {
        return false;
    }

    let ns_string = unsafe { NSString::alloc(nil).init_str(trimmed_url) };
    if ns_string == nil {
        return false;
    }

    let ns_url: id = msg_send![class!(NSURL), URLWithString: ns_string];
    let _: () = msg_send![ns_string, release];

    if ns_url == nil {
        return false;
    }

    let request: id = msg_send![class!(NSURLRequest), requestWithURL: ns_url];
    if request == nil {
        return false;
    }

    let _: id = msg_send![webview, loadRequest: request];
    true
}

/// Updates a `WKWebView` frame.
#[allow(dead_code)]
pub(crate) unsafe fn set_frame(webview: id, frame: NSRect) {
    if webview == nil {
        return;
    }

    let _: () = msg_send![webview, setFrame: frame];
}

/// Removes a `WKWebView` from its parent view.
#[allow(dead_code)]
pub(crate) unsafe fn remove_from_superview(webview: id) {
    if webview == nil {
        return;
    }

    let _: () = msg_send![webview, removeFromSuperview];
}

/// Shows or hides a `WKWebView`.
#[allow(dead_code)]
pub(crate) unsafe fn set_hidden(webview: id, hidden: bool) {
    if webview == nil {
        return;
    }

    let hidden_flag = if hidden { 1_i8 } else { 0_i8 };
    let _: () = msg_send![webview, setHidden: hidden_flag];
}

/// Releases a retained `WKWebView`.
#[allow(dead_code)]
pub(crate) unsafe fn release(webview: id) {
    if webview == nil {
        return;
    }

    let _: () = msg_send![webview, release];
}
