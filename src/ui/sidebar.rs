use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::{App, InputMode, Panel, SidebarItem};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.active_panel == Panel::Sidebar;

    let block = Block::default()
        .title(" Projects ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        }));

    let items: Vec<ListItem> = app
        .sidebar_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let (prefix, name, style) = match item {
                SidebarItem::Project(pi) => {
                    let project = &app.workspace.projects[*pi];
                    let expanded = app.expanded_projects.contains(pi);
                    let icon = if expanded { "▼ " } else { "▶ " };
                    (
                        format!("{}", icon),
                        project.name.clone(),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )
                }
                SidebarItem::Collection(pi, ci) => {
                    let collection = &app.workspace.projects[*pi].collections[*ci];
                    let expanded = app.expanded_collections.contains(&(*pi, *ci));
                    let icon = if expanded { "▼ " } else { "▶ " };
                    (
                        format!("  {}", icon),
                        collection.name.clone(),
                        Style::default().fg(Color::Blue),
                    )
                }
                SidebarItem::Request(pi, ci, ri) => {
                    let request = &app.workspace.projects[*pi].collections[*ci].requests[*ri];
                    let method_color = match request.method {
                        crate::models::HttpMethod::GET => Color::Green,
                        crate::models::HttpMethod::POST => Color::Yellow,
                        crate::models::HttpMethod::PUT => Color::Blue,
                        crate::models::HttpMethod::PATCH => Color::Magenta,
                        crate::models::HttpMethod::DELETE => Color::Red,
                    };
                    (
                        format!("    "),
                        format!("{} {}", request.method, request.name),
                        Style::default().fg(method_color),
                    )
                }
            };

            let is_selected = idx == app.sidebar_selected && is_focused;
            let final_style = if is_selected {
                style.bg(Color::DarkGray)
            } else {
                style
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::styled(name, final_style),
            ]))
        })
        .collect();

    let list = List::new(items).block(block);

    let mut state = ListState::default();
    state.select(Some(app.sidebar_selected));

    frame.render_stateful_widget(list, area, &mut state);

    // Render help at bottom
    if is_focused && app.input_mode == InputMode::Normal {
        let help_text = "n:new N:project d:del ↵:open";
        let help_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - 1,
            width: area.width.saturating_sub(2).min(help_text.len() as u16),
            height: 1,
        };
        frame.render_widget(
            ratatui::widgets::Paragraph::new(help_text)
                .style(Style::default().fg(Color::DarkGray)),
            help_area,
        );
    }
}
