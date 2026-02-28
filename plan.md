# WKWebView in GPUI (macOS-only) — Fastest Path Plan

## Goal

Get **any embedded web content visible inside a GPUI window on macOS** as fast as possible, then iterate toward a reusable API for CAD preview workflows.

Success criteria for phase 1:

- Launch app
- Show a GPUI window
- Render a native `WKWebView` in that window
- Load `https://example.com` (or inline HTML)
- Resize correctly with window

---

## Scope (intentionally narrow)

### In scope

- macOS only
- native `WKWebView` hosted as an `NSView` subview
- hardcoded URL / HTML for first visible result
- simple lifecycle: create, attach, resize, destroy

### Out of scope (for now)

- Linux/Windows
- sandbox/security hardening
- JS bridge
- navigation controls
- clipboard/input edge cases
- extension API exposure
- polished GPUI element API

---

## Difficulty estimate

- **First visible webview:** 0.5–1.5 days
- **Usable internal API (create/show/hide/load URL):** 2–4 days
- **Stable enough for iteration (resize/focus/lifecycle bugs addressed):** 1–2 weeks

Risk level: **medium** (Objective-C runtime integration and view layering details)

---

## Architecture sketch (fastest implementation)

1. Add a tiny macOS-only wrapper around `WKWebView` in `gpui_macos`.
2. Attach the webview as a child of the existing window content view.
3. Keep it full-frame to simplify layout.
4. Trigger URL load on creation.
5. Update frame on window resize callback.

No new cross-platform abstraction initially unless required to compile cleanly.

---

## TODO Set 0 — Preflight (1–2 hours)

- [ ] Confirm current macOS window/view ownership points in `gpui_macos` window code.
- [ ] Decide where to store per-window webview handle (`Option<id>` field on window state).
- [ ] Confirm build deps for WebKit framework linkage are already available or add minimal linkage.
- [ ] Add compile guards (`#[cfg(target_os = "macos")]`) for all new code.

Deliverable: written notes in code comments + compile-ready scaffolding.

---

## TODO Set 1 — “Get something on screen” spike (same day)

- [x] Add `webview.rs` in `gpui_macos` with:
  - `create_wkwebview(frame: NSRect) -> id`
  - `load_url(webview: id, url: &str)`
  - `set_frame(webview: id, frame: NSRect)`
  - `remove_from_superview(webview: id)`
- [x] In macOS window creation path:
  - create `WKWebView`
  - add as subview to content view
  - set autoresizing mask or manual resize hook
  - load `https://example.com`
- [x] Add temporary command-line or env flag (`ZED_EXPERIMENTAL_WEBVIEW=1`) to enable.
- [x] Ensure app still starts with feature off.
- [x] Add window-level APIs:
  - `Window::set_webview_bounds(Option<Bounds<Pixels>>)`
  - `Window::load_webview_url(&str)`
- [x] Add lazy webview creation on first bounds/url call (`ensure_webview`).
- [x] Add `gpui` split example (`webview_split`) with GPUI left pane + webview right pane.
- [x] Make bounds ownership layout-driven for the example path (avoid ratio-based resize rewriting in platform callback).

Deliverable: web page visible inside a GPUI window on macOS.

---

## TODO Set 2 — Stabilize feedback loop (1–2 days)

- [ ] Resize correctness:
  - update webview frame on window resize
  - verify retina scaling looks normal
- [ ] Lifecycle:
  - detach webview on window close
  - avoid leaked references
- [ ] Z-order:
  - ensure webview appears above/below intended GPUI content consistently
- [ ] Focus/input sanity:
  - click into webview, type in page inputs
  - click back to GPUI areas and verify key handling returns
- [ ] Logging:
  - emit concise logs for create/load/fail/destroy

Deliverable: repeatable local loop where web content reliably appears and updates.

Status:

- Builds and runs via `cargo run -p gpui --example webview_split`.
- Runtime visual verification (flicker/layering/focus) is still a manual pass to complete.

---

## TODO Set 3 — Minimal internal API (2–4 days)

- [ ] Introduce tiny API surface in macOS backend:
  - `open_webview(url: String)`
  - `close_webview()`
  - `set_webview_bounds(Bounds<Pixels>)`
- [x] Keep API intentionally private/internal first.
- [ ] Add an experiment-only action/command to open webview for rapid testing.
- [ ] Support `load_html_string` to remove network dependency during tests.
- [ ] Add element-level embedding path (`div().child(webview(...))`) and route layout bounds through window/platform webview bounds updates.

Deliverable: easy manual trigger to open/close/change URL without code rewiring each time.

---

## TODO Set 4 — CAD-oriented iteration hooks (optional, after visibility)

- [ ] Add `localhost` URL support by default for sidecar viewer.
- [ ] Add `reload` action.
- [ ] Add very small JS eval hook (optional) for camera reset/test pings.
- [ ] Add basic navigation state callbacks (loading/error) for user feedback.

Deliverable: practical host shell for OCP CAD Viewer-like sidecar.

---

## Suggested implementation order (fast loop)

1. Hardcode webview creation + `example.com` ✅
2. Make it resizable and close cleanly ✅
3. Add enable flag ✅
4. Add split example with window-level bounds API ✅
5. Add action to toggle open/close
6. Implement element trait + `div().child(webview(...))` embedding path
7. Replace URL with local sidecar URL

---

## Testing checklist (manual)

- [ ] Feature off: app behavior unchanged
- [ ] Feature on: webview appears on launch
- [ ] Window resize keeps webview fitted
- [ ] Split example: GPUI left pane + webview right pane remains aligned during resize
- [ ] Split example: no obvious flicker/layering artifacts while resizing
- [ ] Close/reopen window does not crash
- [ ] Keyboard focus can move webview <-> GPUI
- [ ] Loading bad URL shows non-crashing failure state

---

## Known risks / likely friction

- Objective-C ownership mistakes (retain/release) causing leaks or crashes
- Focus and key event routing conflicts between NSView and GPUI view
- Layering/compositing artifacts with Metal-backed GPUI content
- Event ordering on resize causing flicker

Mitigation: keep scope tiny, add logs, and avoid introducing public abstractions too early.

---

## Fastest “definition of done”

- A branch where setting one env var opens a macOS window with a visible `WKWebView` loading a known URL, resizes correctly, and closes cleanly.

That gives the shortest feedback loop and unblocks immediate CAD viewer sidecar experiments.
