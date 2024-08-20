use debugger_panel::{DebugPanel, ToggleFocus};
use gpui::AppContext;
use workspace::{StartDebugger, Workspace};

pub mod debugger_panel;
mod debugger_panel_item;
mod elements;

pub fn init(cx: &mut AppContext) {
    cx.observe_new_views(|workspace: &mut Workspace, _| {
        workspace.register_action(|workspace, _: &ToggleFocus, cx| {
            workspace.toggle_panel_focus::<DebugPanel>(cx);
        });
        workspace.register_action(|workspace: &mut Workspace, _: &StartDebugger, cx| {
            tasks_ui::toggle_modal(workspace, cx, task::TaskType::Debug).detach();
        });
    })
    .detach();
}
