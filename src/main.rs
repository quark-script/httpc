mod app;
mod events;
mod http;
mod models;
mod storage;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use app::App;
use http::HttpClient;
use storage::JsonStore;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load workspace
    let store = JsonStore::new()?;
    let workspace = store.load_workspace()?;

    // Create app
    let mut app = App::new(workspace);

    // Expand first project and collection by default if exists
    if !app.workspace.projects.is_empty() {
        app.expanded_projects.push(0);
        if !app.workspace.projects[0].collections.is_empty() {
            app.expanded_collections.push((0, 0));
        }
        app.rebuild_sidebar_items();
    }

    // Create HTTP client
    let http_client = HttpClient::new();

    // Channel for async HTTP responses
    let (tx, mut rx) = mpsc::channel::<Result<models::HttpResponse, String>>(1);

    // Run the app
    let result = run_app(&mut terminal, &mut app, &http_client, tx, &mut rx, &store).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    http_client: &HttpClient,
    tx: mpsc::Sender<Result<models::HttpResponse, String>>,
    rx: &mut mpsc::Receiver<Result<models::HttpResponse, String>>,
    store: &JsonStore,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| {
            ui::render(frame, app);
        })?;

        // Check for HTTP response
        if let Ok(result) = rx.try_recv() {
            app.is_loading = false;
            match result {
                Ok(response) => {
                    app.response = Some(response);
                    app.error_message = None;
                    app.response_scroll = 0;
                }
                Err(err) => {
                    app.error_message = Some(err);
                    app.response = None;
                }
            }
        }

        // Poll for events
        if let Some(event) = events::poll_event(Duration::from_millis(100))? {
            match event {
                Event::Key(key) => {
                    // Check if we need to send a request before handling the key
                    let should_send = app.is_loading && app.current_request.is_some();

                    events::handle_key_event(app, key);

                    // Send HTTP request if triggered
                    if should_send {
                        if let Some(ref request) = app.current_request {
                            let request = request.clone();
                            // Update the URL from input
                            let mut req = request.clone();
                            req.url = app.url_input.clone();
                            req.body = app.body_input.clone();
                            req.headers = app.headers_list.iter().cloned().collect();

                            let tx = tx.clone();
                            let client = http_client.client.clone();

                            tokio::spawn(async move {
                                let http_client = HttpClient { client };
                                let result = http_client.execute(&req).await;
                                let _ = tx.send(result.map_err(|e| e.to_string())).await;
                            });
                        }
                    }

                    // Save workspace if needed
                    if app.needs_save {
                        if let Err(e) = store.save_workspace(&app.workspace) {
                            app.error_message = Some(format!("Failed to save: {}", e));
                        } else {
                            app.needs_save = false;
                        }
                    }

                    if app.should_quit {
                        break;
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal will redraw on next loop
                }
                _ => {}
            }
        }
    }

    Ok(())
}
