use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::{App, DialogType, EditorField, EditorTab, InputMode, Panel, SidebarItem};

pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    // Handle dialogs first
    if app.dialog.is_some() {
        handle_dialog_input(app, key);
        return;
    }

    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::Editing => handle_editing_mode(app, key),
    }
}

fn handle_dialog_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.dialog = None;
            app.dialog_input.clear();
        }
        KeyCode::Enter => {
            if !app.dialog_input.is_empty() {
                match app.dialog {
                    Some(DialogType::NewProject) => {
                        app.add_project(app.dialog_input.clone());
                    }
                    Some(DialogType::NewCollection) => {
                        if let Some(pi) = app.get_selected_project_index() {
                            app.add_collection(pi, app.dialog_input.clone());
                        }
                    }
                    Some(DialogType::NewRequest) => {
                        if let Some((pi, ci)) = app.get_selected_collection_index() {
                            app.add_request(pi, ci, app.dialog_input.clone());
                        }
                    }
                    Some(DialogType::DeleteConfirm) => {
                        if app.dialog_input.to_lowercase() == "y" {
                            app.delete_selected();
                        }
                    }
                    Some(DialogType::RenameItem) => {
                        // TODO: Implement rename
                    }
                    None => {}
                }
            }
            app.dialog = None;
            app.dialog_input.clear();
        }
        KeyCode::Char(c) => {
            app.dialog_input.push(c);
        }
        KeyCode::Backspace => {
            app.dialog_input.pop();
        }
        _ => {}
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Tab => {
            app.active_panel = app.active_panel.next();
        }
        KeyCode::BackTab => {
            app.active_panel = app.active_panel.prev();
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.save_current_request();
        }
        _ => match app.active_panel {
            Panel::Sidebar => handle_sidebar_keys(app, key),
            Panel::RequestEditor => handle_editor_keys(app, key),
            Panel::ResponseViewer => handle_response_keys(app, key),
        },
    }
}

fn handle_sidebar_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.sidebar_selected > 0 {
                app.sidebar_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.sidebar_selected < app.sidebar_items.len().saturating_sub(1) {
                app.sidebar_selected += 1;
            }
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            if let Some(&item) = app.sidebar_items.get(app.sidebar_selected) {
                match item {
                    SidebarItem::Project(pi) => {
                        app.toggle_project_expand(pi);
                    }
                    SidebarItem::Collection(pi, ci) => {
                        app.toggle_collection_expand(pi, ci);
                    }
                    SidebarItem::Request(pi, ci, ri) => {
                        app.select_request(pi, ci, ri);
                        app.active_panel = Panel::RequestEditor;
                    }
                }
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if let Some(&item) = app.sidebar_items.get(app.sidebar_selected) {
                match item {
                    SidebarItem::Project(pi) => {
                        if app.expanded_projects.contains(&pi) {
                            app.toggle_project_expand(pi);
                        }
                    }
                    SidebarItem::Collection(pi, ci) => {
                        if app.expanded_collections.contains(&(pi, ci)) {
                            app.toggle_collection_expand(pi, ci);
                        }
                    }
                    SidebarItem::Request(_, _, _) => {}
                }
            }
        }
        KeyCode::Char('n') => {
            if let Some(&item) = app.sidebar_items.get(app.sidebar_selected) {
                match item {
                    SidebarItem::Project(_) => {
                        app.dialog = Some(DialogType::NewCollection);
                    }
                    SidebarItem::Collection(_, _) => {
                        app.dialog = Some(DialogType::NewRequest);
                    }
                    SidebarItem::Request(_, _, _) => {
                        app.dialog = Some(DialogType::NewRequest);
                    }
                }
            } else {
                app.dialog = Some(DialogType::NewProject);
            }
        }
        KeyCode::Char('N') => {
            app.dialog = Some(DialogType::NewProject);
        }
        KeyCode::Char('d') => {
            if !app.sidebar_items.is_empty() {
                app.dialog = Some(DialogType::DeleteConfirm);
            }
        }
        _ => {}
    }
}

fn handle_editor_keys(app: &mut App, key: KeyEvent) {
    if app.current_request.is_none() {
        return;
    }

    match key.code {
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Signal to send request (handled in main loop)
            app.is_loading = true;
        }
        KeyCode::Char('1') => {
            app.editor_tab = EditorTab::Url;
        }
        KeyCode::Char('2') => {
            app.editor_tab = EditorTab::Headers;
        }
        KeyCode::Char('3') => {
            app.editor_tab = EditorTab::Body;
        }
        KeyCode::Left => {
            app.editor_tab = app.editor_tab.prev();
        }
        KeyCode::Right => {
            app.editor_tab = app.editor_tab.next();
        }
        KeyCode::Char('m') => {
            app.cycle_method();
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            app.input_mode = InputMode::Editing;
            match app.editor_tab {
                EditorTab::Url => app.editor_field = EditorField::Url,
                EditorTab::Headers => app.editor_field = EditorField::HeaderKey,
                EditorTab::Body => app.editor_field = EditorField::Body,
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.editor_tab == EditorTab::Headers {
                // Navigate headers list
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.editor_tab == EditorTab::Headers {
                // Navigate headers list
            }
        }
        KeyCode::Char('x') => {
            if app.editor_tab == EditorTab::Headers && app.editing_header_index.is_some() {
                if let Some(idx) = app.editing_header_index {
                    app.remove_header(idx);
                    app.editing_header_index = None;
                }
            }
        }
        _ => {}
    }
}

fn handle_editing_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.save_current_request();
        }
        KeyCode::Tab => {
            match app.editor_field {
                EditorField::HeaderKey => app.editor_field = EditorField::HeaderValue,
                EditorField::HeaderValue => {
                    app.add_header();
                    app.editor_field = EditorField::HeaderKey;
                }
                _ => {}
            }
        }
        KeyCode::Enter => {
            match app.editor_field {
                EditorField::HeaderValue => {
                    app.add_header();
                }
                EditorField::Body => {
                    app.body_input.push('\n');
                }
                _ => {}
            }
        }
        KeyCode::Char(c) => {
            match app.editor_field {
                EditorField::Url => app.url_input.push(c),
                EditorField::HeaderKey => app.header_key_input.push(c),
                EditorField::HeaderValue => app.header_value_input.push(c),
                EditorField::Body => app.body_input.push(c),
                EditorField::Method => {}
            }
        }
        KeyCode::Backspace => {
            match app.editor_field {
                EditorField::Url => { app.url_input.pop(); }
                EditorField::HeaderKey => { app.header_key_input.pop(); }
                EditorField::HeaderValue => { app.header_value_input.pop(); }
                EditorField::Body => { app.body_input.pop(); }
                EditorField::Method => {}
            }
        }
        _ => {}
    }
}

fn handle_response_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.response_scroll = app.response_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.response_scroll = app.response_scroll.saturating_add(1);
        }
        KeyCode::PageUp => {
            app.response_scroll = app.response_scroll.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.response_scroll = app.response_scroll.saturating_add(10);
        }
        KeyCode::Home | KeyCode::Char('g') => {
            app.response_scroll = 0;
        }
        KeyCode::End | KeyCode::Char('G') => {
            app.response_scroll = u16::MAX;
        }
        _ => {}
    }
}
