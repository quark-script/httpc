use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, DialogType, InputMode};
use crate::ui::{request_editor, response_viewer, sidebar};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.size();

    // Main layout: sidebar left, editor+response right
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(size);

    // Right side: editor top, response bottom
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    // Render components
    sidebar::render(frame, app, main_chunks[0]);
    request_editor::render(frame, app, right_chunks[0]);
    response_viewer::render(frame, app, right_chunks[1]);

    // Render status bar at bottom
    render_status_bar(frame, app, size);

    // Render dialog if active
    if app.dialog.is_some() {
        render_dialog(frame, app, size);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_area = Rect {
        x: 0,
        y: area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };

    let mode_text = match app.input_mode {
        InputMode::Normal => "NORMAL",
        InputMode::Editing => "EDITING",
    };

    let mode_style = match app.input_mode {
        InputMode::Normal => Style::default().fg(Color::Black).bg(Color::Cyan),
        InputMode::Editing => Style::default().fg(Color::Black).bg(Color::Yellow),
    };

    let save_indicator = if app.needs_save { " [modified]" } else { "" };

    let request_name = app
        .current_request
        .as_ref()
        .map(|r| r.name.as_str())
        .unwrap_or("No request selected");

    let status = Line::from(vec![
        Span::styled(format!(" {} ", mode_text), mode_style),
        Span::raw(" "),
        Span::styled(request_name, Style::default().fg(Color::White)),
        Span::styled(save_indicator, Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(
            "Tab:panel q:quit Ctrl+R:send Ctrl+S:save",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let status_bar = Paragraph::new(status)
        .style(Style::default().bg(Color::Black));

    frame.render_widget(status_bar, status_area);
}

fn render_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let dialog = app.dialog.as_ref().unwrap();

    let title = match dialog {
        DialogType::NewProject => "New Project",
        DialogType::NewCollection => "New Collection",
        DialogType::NewRequest => "New Request",
        DialogType::DeleteConfirm => "Confirm Delete",
        DialogType::RenameItem => "Rename",
    };

    let prompt = match dialog {
        DialogType::DeleteConfirm => "Delete this item? (y/N): ",
        _ => "Name: ",
    };

    let width = 50u16;
    let height = 5u16;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let dialog_area = Rect {
        x,
        y,
        width,
        height,
    };

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(block.clone(), dialog_area);

    let inner = block.inner(dialog_area);

    let input = Paragraph::new(Line::from(vec![
        Span::styled(prompt, Style::default().fg(Color::DarkGray)),
        Span::styled(&app.dialog_input, Style::default().fg(Color::White)),
        Span::styled("│", Style::default().fg(Color::Cyan)),
    ]));

    frame.render_widget(input, inner);

    let help_area = Rect {
        x: inner.x,
        y: inner.y + 2,
        width: inner.width,
        height: 1,
    };

    let help = Paragraph::new("Enter: confirm | Esc: cancel")
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(help, help_area);
}
