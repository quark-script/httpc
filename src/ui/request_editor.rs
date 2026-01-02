use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::app::{App, EditorField, EditorTab, InputMode, Panel};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.active_panel == Panel::RequestEditor;

    let block = Block::default()
        .title(" Request ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        }));

    frame.render_widget(block, area);

    let inner = Block::default().borders(Borders::ALL).inner(area);

    if app.current_request.is_none() {
        let placeholder = Paragraph::new("Select a request from the sidebar")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(placeholder, inner);
        return;
    }

    let request = app.current_request.as_ref().unwrap();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Method + URL
            Constraint::Length(2), // Tabs
            Constraint::Min(5),    // Content
        ])
        .split(inner);

    // Method + URL bar
    render_url_bar(frame, app, request, chunks[0], is_focused);

    // Tabs
    render_tabs(frame, app, chunks[1], is_focused);

    // Tab content
    match app.editor_tab {
        EditorTab::Url => render_url_details(frame, app, chunks[2]),
        EditorTab::Headers => render_headers(frame, app, chunks[2], is_focused),
        EditorTab::Body => render_body(frame, app, chunks[2], is_focused),
    }
}

fn render_url_bar(
    frame: &mut Frame,
    app: &App,
    request: &crate::models::Request,
    area: Rect,
    is_focused: bool,
) {
    let method_color = match request.method {
        crate::models::HttpMethod::GET => Color::Green,
        crate::models::HttpMethod::POST => Color::Yellow,
        crate::models::HttpMethod::PUT => Color::Blue,
        crate::models::HttpMethod::PATCH => Color::Magenta,
        crate::models::HttpMethod::DELETE => Color::Red,
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(20)])
        .split(area);

    let is_editing = app.input_mode == InputMode::Editing && app.editor_field == EditorField::Url;

    // Method
    let method_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(if is_focused { "[m]" } else { "" });
    let method = Paragraph::new(format!(" {} ", request.method))
        .style(Style::default().fg(method_color).add_modifier(Modifier::BOLD))
        .block(method_block);
    frame.render_widget(method, chunks[0]);

    // URL
    let url_style = if is_editing {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let url_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_editing {
            Color::Cyan
        } else {
            Color::DarkGray
        }))
        .title(if is_focused { "[e]dit" } else { "" });

    let cursor_suffix = if is_editing { "│" } else { "" };
    let url = Paragraph::new(format!("{}{}", app.url_input, cursor_suffix))
        .style(url_style)
        .block(url_block);
    frame.render_widget(url, chunks[1]);
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect, _is_focused: bool) {
    let titles = vec!["1:URL", "2:Headers", "3:Body"];
    let selected = match app.editor_tab {
        EditorTab::Url => 0,
        EditorTab::Headers => 1,
        EditorTab::Body => 2,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(" | "));

    frame.render_widget(tabs, area);
}

fn render_url_details(frame: &mut Frame, app: &App, area: Rect) {
    let request = app.current_request.as_ref().unwrap();
    let text = format!(
        "Full URL: {}\n\nMethod: {}\nName: {}",
        app.url_input, request.method, request.name
    );
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_headers(frame: &mut Frame, app: &App, area: Rect, _is_focused: bool) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    // Headers list
    let headers_text: Vec<Line> = app
        .headers_list
        .iter()
        .enumerate()
        .map(|(idx, (k, v))| {
            let selected = app.editing_header_index == Some(idx);
            let style = if selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            Line::from(vec![
                Span::styled(format!("{}: ", k), style.fg(Color::Cyan)),
                Span::styled(v.clone(), style.fg(Color::White)),
            ])
        })
        .collect();

    let headers_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Headers ");

    let headers = if headers_text.is_empty() {
        Paragraph::new("No headers. Press [e] to add.")
            .style(Style::default().fg(Color::DarkGray))
            .block(headers_block)
    } else {
        Paragraph::new(headers_text).block(headers_block)
    };

    frame.render_widget(headers, chunks[0]);

    // Header input
    let is_editing_key = app.input_mode == InputMode::Editing
        && app.editor_field == EditorField::HeaderKey;
    let is_editing_value = app.input_mode == InputMode::Editing
        && app.editor_field == EditorField::HeaderValue;

    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    let key_style = if is_editing_key {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let value_style = if is_editing_value {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let cursor = "│";

    let key_input = Paragraph::new(format!(
        "{}{}",
        app.header_key_input,
        if is_editing_key { cursor } else { "" }
    ))
    .style(key_style)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if is_editing_key {
                Color::Cyan
            } else {
                Color::DarkGray
            }))
            .title(" Key "),
    );

    let value_input = Paragraph::new(format!(
        "{}{}",
        app.header_value_input,
        if is_editing_value { cursor } else { "" }
    ))
    .style(value_style)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if is_editing_value {
                Color::Cyan
            } else {
                Color::DarkGray
            }))
            .title(" Value "),
    );

    frame.render_widget(key_input, input_chunks[0]);
    frame.render_widget(value_input, input_chunks[1]);
}

fn render_body(frame: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let is_editing = app.input_mode == InputMode::Editing && app.editor_field == EditorField::Body;

    let style = if is_editing {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let cursor = if is_editing { "│" } else { "" };

    let body = Paragraph::new(format!("{}{}", app.body_input, cursor))
        .style(style)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if is_editing {
                    Color::Cyan
                } else {
                    Color::DarkGray
                }))
                .title(format!(
                    " Body (JSON) {} ",
                    if is_focused { "[e]dit" } else { "" }
                )),
        );

    frame.render_widget(body, area);
}
