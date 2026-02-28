use crate::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    Pixels, Style, StyleRefinement, Styled, Window, hash,
};
use refineable::Refineable;

/// Construct an embedded platform webview element.
pub fn webview(url: impl Into<String>) -> Webview {
    Webview {
        url: url.into(),
        visible: true,
        style: StyleRefinement::default(),
    }
}

/// An embedded platform webview element.
pub struct Webview {
    url: String,
    visible: bool,
    style: StyleRefinement,
}

impl Webview {
    /// Set whether this webview should be visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Replace the current URL.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }
}

impl IntoElement for Webview {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Styled for Webview {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

type WebviewState = bool;

impl Element for Webview {
    type RequestLayoutState = (Style, u64);
    type PrepaintState = WebviewState;

    fn id(&self) -> Option<ElementId> {
        // Ensure this element always receives a global id and therefore stable element state.
        Some(ElementId::CodeLocation(*core::panic::Location::caller()))
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.refine(&self.style);
        let layout_id = window.request_layout(style.clone(), [], cx);

        let global_id = global_id.expect("webview elements must always have a global id");
        let native_id = hash(global_id);

        (layout_id, (style, native_id))
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<crate::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let global_id = id.expect("webview elements must always have a global id");
        let native_id = request_layout.1;

        let was_visible_in_previous_frame =
            window.with_element_state(global_id, |previous: Option<WebviewState>, _| {
                let state = previous.unwrap_or(false);
                (state, state)
            });

        let content_mask = window.content_mask();
        let clipped_bounds = bounds.intersect(&content_mask.bounds);
        let should_show = self.visible && !clipped_bounds.is_empty();

        let rem_size = window.rem_size();
        let corner_radii = request_layout.0.corner_radii.to_pixels(rem_size);
        let max_corner_radius = Pixels::from(
            corner_radii
                .top_left
                .max(corner_radii.top_right)
                .max(corner_radii.bottom_left)
                .max(corner_radii.bottom_right),
        );

        if should_show {
            window.upsert_webview(
                native_id,
                clipped_bounds,
                &self.url,
                true,
                max_corner_radius,
            );
        } else if was_visible_in_previous_frame {
            window.destroy_webview(native_id);
        }

        should_show
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<crate::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        // Persist lifecycle state for the next frame.
        if let Some(global_id) = id {
            let state_to_store = prepaint.clone();
            window.with_element_state(global_id, |_previous: Option<WebviewState>, _| {
                ((), state_to_store)
            });
        }
    }
}
