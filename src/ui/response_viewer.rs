use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use crate::app::{App, Panel};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.active_panel == Panel::ResponseViewer;

    let block = Block::default()
        .title(" Response ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        }));

    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);

    if app.is_loading {
        let loading = Paragraph::new("Sending request...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default());
        frame.render_widget(loading, inner);
        return;
    }

    if let Some(ref error) = app.error_message {
        let error_text = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true })
            .block(Block::default());
        frame.render_widget(error_text, inner);
        return;
    }

    if app.response.is_none() {
        let placeholder = Paragraph::new("Press Ctrl+R to send a request")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(placeholder, inner);
        return;
    }

    let response = app.response.as_ref().unwrap();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(5)])
        .split(inner);

    // Status bar
    render_status_bar(frame, response, chunks[0]);

    // Body
    render_body(frame, app, response, chunks[1], is_focused);
}

fn render_status_bar(frame: &mut Frame, response: &crate::models::HttpResponse, area: Rect) {
    let status_color = if response.is_success() {
        Color::Green
    } else if response.is_error() {
        Color::Red
    } else {
        Color::Yellow
    };

    let status_line = Line::from(vec![
        Span::styled(
            format!(" {} {} ", response.status, response.status_text),
            Style::default()
                .fg(Color::Black)
                .bg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{}ms", response.time_ms),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" "),
        Span::styled(
            response.formatted_size(),
            Style::default().fg(Color::Magenta),
        ),
    ]);

    let status = Paragraph::new(status_line);
    frame.render_widget(status, area);
}

fn render_body(
    frame: &mut Frame,
    app: &App,
    response: &crate::models::HttpResponse,
    area: Rect,
    is_focused: bool,
) {
    let formatted = response.formatted_body();
    let lines: Vec<&str> = formatted.lines().collect();
    let total_lines = lines.len();

    let scroll = app.response_scroll as usize;
    let visible_height = area.height.saturating_sub(2) as usize;

    let scroll = scroll.min(total_lines.saturating_sub(visible_height));

    let visible_lines: Vec<Line> = lines
        .iter()
        .skip(scroll)
        .take(visible_height)
        .map(|line| {
            // Simple JSON syntax highlighting
            let colored = colorize_json_line(line);
            colored
        })
        .collect();

    let body_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(format!(
            " Body {} ",
            if is_focused { "[j/k:scroll]" } else { "" }
        ));

    let body = Paragraph::new(visible_lines).block(body_block);

    frame.render_widget(body, area);

    // Scrollbar
    if total_lines > visible_height {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(scroll)
            .viewport_content_length(visible_height);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(&ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn colorize_json_line(line: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '"' {
            // Find the closing quote
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1; // include closing quote

            let s: String = chars[start..i.min(chars.len())].iter().collect();

            // Check if it's a key (followed by :)
            let remaining: String = chars[i..].iter().collect();
            let trimmed = remaining.trim_start();
            if trimmed.starts_with(':') {
                spans.push(Span::styled(s, Style::default().fg(Color::Cyan)));
            } else {
                spans.push(Span::styled(s, Style::default().fg(Color::Green)));
            }
        } else if c == ':' || c == ',' || c == '{' || c == '}' || c == '[' || c == ']' {
            spans.push(Span::styled(
                c.to_string(),
                Style::default().fg(Color::DarkGray),
            ));
            i += 1;
        } else if c.is_numeric() || c == '-' || c == '.' {
            let start = i;
            while i < chars.len()
                && (chars[i].is_numeric() || chars[i] == '.' || chars[i] == '-' || chars[i] == 'e' || chars[i] == 'E')
            {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::styled(s, Style::default().fg(Color::Yellow)));
        } else if line[i..].starts_with("true") {
            spans.push(Span::styled(
                "true".to_string(),
                Style::default().fg(Color::Magenta),
            ));
            i += 4;
        } else if line[i..].starts_with("false") {
            spans.push(Span::styled(
                "false".to_string(),
                Style::default().fg(Color::Magenta),
            ));
            i += 5;
        } else if line[i..].starts_with("null") {
            spans.push(Span::styled(
                "null".to_string(),
                Style::default().fg(Color::Red),
            ));
            i += 4;
        } else {
            spans.push(Span::raw(c.to_string()));
            i += 1;
        }
    }

    Line::from(spans)
}
