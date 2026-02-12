use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::app::App;
use crate::overlays;
use crate::panels::{PanelId, TIMER_MIN_WIDTH};

/// Layout regions for timer and tasks panels
pub struct AppLayout {
    pub timer: Option<Rect>,
    pub tasks: Option<Rect>,
}

pub fn create_layout(area: Rect, app: &App) -> AppLayout {
    let (timer_area, tasks_area) = if app.tasks_visible {
        let content_chunks =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

        if content_chunks[0].width < TIMER_MIN_WIDTH {
            if app.focused_panel == PanelId::Timer {
                (Some(area), None)
            } else {
                (None, Some(area))
            }
        } else {
            (Some(content_chunks[0]), Some(content_chunks[1]))
        }
    } else {
        (Some(area), None)
    };

    AppLayout {
        timer: timer_area,
        tasks: tasks_area,
    }
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = create_layout(frame.area(), app);

    if let Some(timer_area) = layout.timer {
        app.timer_panel.render(
            frame,
            timer_area,
            app.focused_panel == PanelId::Timer,
            &app.timer,
            app.task_manager.active_task(),
        );
    }

    if let Some(tasks_area) = layout.tasks {
        app.tasks_panel.render(
            frame,
            tasks_area,
            app.focused_panel == PanelId::Tasks,
            &app.task_manager,
        );
    }

    // Render overlays
    if let Some(ref message) = app.error_message {
        overlays::render_error_overlay(frame, message);
    } else if let Some(ref input) = app.task_input_overlay {
        input.render(frame);
    } else if let Some(ref sync) = app.sync_overlay {
        sync.render(frame);
    } else if app.shortcuts_visible {
        let panel_name = match app.focused_panel {
            PanelId::Timer => "Timer",
            PanelId::Tasks => "Tasks",
        };
        let shortcuts = app.focused_shortcuts();
        overlays::render_help_overlay(frame, panel_name, &shortcuts);
    }
}
