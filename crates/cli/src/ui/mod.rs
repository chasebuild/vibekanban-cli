//! UI components and rendering.

pub mod components;
pub mod views;

use ratatui::Frame;

use crate::app::App;

/// Render the UI based on current application state.
pub fn render(frame: &mut Frame, app: &App) {
    use crate::app::View;

    match app.view {
        View::Projects => views::projects::render(frame, app),
        View::Tasks => views::tasks::render(frame, app),
        View::Workspaces => views::workspaces::render(frame, app),
        View::WorkspaceDetail => views::workspace_detail::render(frame, app),
        View::CreateTask => views::create_task::render(frame, app),
        View::Help => views::help::render(frame, app),
    }
}
