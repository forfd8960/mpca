//! Terminal User Interface for interactive planning mode.
//!
//! Provides a ratatui-based TUI for interactively planning features with Claude.
//! Users can review generated specs, edit content, and approve or regenerate plans.

use anyhow::{Context, Result};
use claude_agent_sdk_rs::{ClaudeAgentOptions, ClaudeClient, ContentBlock, Message};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::stream::StreamExt;
use mpca_core::AgentRuntime;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::io;
use tokio::sync::mpsc;

/// A message in the chat history
#[derive(Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Application state for the planning TUI
struct PlanningApp {
    /// Feature slug being planned
    feature_name: String,

    /// Chat message history
    messages: Vec<ChatMessage>,

    /// Current input text being typed
    input: String,

    /// Current view mode
    view_mode: ViewMode,

    /// Status message
    status: String,

    /// Whether the app should exit
    should_quit: bool,

    /// Whether we're waiting for Claude's response
    waiting_for_response: bool,
}

/// View modes for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    /// Chat with Claude
    Chat,

    /// Help screen
    Help,
}

impl PlanningApp {
    /// Creates a new planning app
    fn new(feature_name: String) -> Self {
        let mut app = Self {
            feature_name: feature_name.clone(),
            messages: Vec::new(),
            input: String::new(),
            view_mode: ViewMode::Chat,
            status: "Type your message and press Enter to send. Press 'q' to quit, 'h' for help"
                .to_string(),
            should_quit: false,
            waiting_for_response: false,
        };

        // Add initial system message
        app.messages.push(ChatMessage {
            role: "system".to_string(),
            content: format!(
                "Welcome to interactive planning for feature: {}\n\
                 Let's work together to create a comprehensive feature plan.",
                feature_name
            ),
        });

        app
    }

    /// Handle keyboard input
    fn handle_input(&mut self, key: KeyCode) -> Option<String> {
        if self.waiting_for_response {
            // Only allow quitting while waiting
            if key == KeyCode::Char('q') {
                self.should_quit = true;
            }
            return None;
        }

        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
                None
            }
            KeyCode::Char('h') => {
                self.view_mode = ViewMode::Help;
                self.status = "Viewing help - press Esc to go back".to_string();
                None
            }
            KeyCode::Esc => {
                if self.view_mode == ViewMode::Help {
                    self.view_mode = ViewMode::Chat;
                    self.status = "Type your message and press Enter to send".to_string();
                }
                None
            }
            KeyCode::Char(c) if self.view_mode == ViewMode::Chat => {
                self.input.push(c);
                None
            }
            KeyCode::Backspace if self.view_mode == ViewMode::Chat => {
                self.input.pop();
                None
            }
            KeyCode::Enter if self.view_mode == ViewMode::Chat && !self.input.is_empty() => {
                let message = self.input.clone();
                self.input.clear();
                self.messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: message.clone(),
                });
                self.waiting_for_response = true;
                self.status = "Waiting for Claude's response...".to_string();
                Some(message)
            }
            _ => None,
        }
    }

    /// Add an assistant message to the chat
    fn add_assistant_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content,
        });
        self.waiting_for_response = false;
        self.status = "Type your message and press Enter to send".to_string();
    }

    /// Add an error message to the chat
    fn add_error(&mut self, error: String) {
        self.messages.push(ChatMessage {
            role: "error".to_string(),
            content: error,
        });
        self.waiting_for_response = false;
        self.status = "Error occurred. Type your message and press Enter to continue".to_string();
    }
}

/// Run the interactive planning TUI
pub async fn run_planning_tui(feature_name: &str, _runtime: &AgentRuntime) -> Result<()> {
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Create app state
    let mut app = PlanningApp::new(feature_name.to_string());

    // Initialize Claude client with planning-appropriate settings
    let initial_prompt = format!(
        "I'm planning a new feature called '{}'. \
         Help me create comprehensive specifications including:\n\
         - Feature overview and goals\n\
         - Requirements (functional and non-functional)\n\
         - Technical design\n\
         - Implementation plan\n\
         - Testing strategy\n\n\
         Let's have an interactive conversation to refine the feature details.",
        feature_name
    );

    // Create channels for bidirectional communication
    let (user_tx, mut user_rx) = mpsc::channel::<String>(32);
    let (agent_tx, mut agent_rx) = mpsc::channel::<String>(32);

    // Spawn agent task
    let agent_task = tokio::spawn(async move {
        // Configure Claude for planning workflow
        let options = ClaudeAgentOptions {
            model: Some("claude-3-5-sonnet-20241022".to_string()),
            max_turns: Some(20),
            ..Default::default()
        };

        let mut client = ClaudeClient::new(options);

        // Connect to Claude
        if let Err(e) = client.connect().await {
            tracing::error!("Failed to connect to Claude: {}", e);
            let _ = agent_tx.send(format!("Error: {}", e)).await;
            return;
        }

        // Send initial planning prompt
        if let Err(e) = client.query(&initial_prompt).await {
            tracing::error!("Failed to send initial prompt: {}", e);
            let _ = agent_tx.send(format!("Error: {}", e)).await;
            return;
        }

        // Process initial response
        if let Err(e) = process_agent_response(&mut client, &agent_tx).await {
            tracing::error!("Failed to process initial response: {}", e);
        }

        // Wait for user messages
        while let Some(message) = user_rx.recv().await {
            if message == "__QUIT__" {
                break;
            }

            if let Err(e) = client.query(&message).await {
                tracing::error!("Failed to send message: {}", e);
                let _ = agent_tx.send(format!("Error: {}", e)).await;
                continue;
            }

            if let Err(e) = process_agent_response(&mut client, &agent_tx).await {
                tracing::error!("Failed to process response: {}", e);
            }
        }

        // Disconnect
        if let Err(e) = client.disconnect().await {
            tracing::error!("Failed to disconnect from Claude: {}", e);
        }
    });

    // Run the event loop
    let result = run_app(&mut terminal, &mut app, user_tx.clone(), &mut agent_rx).await;

    // Signal agent to quit
    let _ = user_tx.send("__QUIT__".to_string()).await;

    // Wait for agent task to complete
    agent_task.await.context("Agent task failed")?;

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    result
}

/// Process Claude's response and send to UI
async fn process_agent_response(
    client: &mut ClaudeClient,
    tx: &mpsc::Sender<String>,
) -> Result<()> {
    let mut response_text = String::new();
    let mut stream = client.receive_messages();

    while let Some(result) = stream.next().await {
        match result? {
            Message::Assistant(msg) => {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        response_text.push_str(&text.text);
                    }
                }
            }
            Message::Result(_) => {
                // End of response
                break;
            }
            _ => continue,
        }
    }

    if !response_text.is_empty() {
        tx.send(response_text).await?;
    }

    Ok(())
}

/// Run the main application loop
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut PlanningApp,
    tx: mpsc::Sender<String>,
    rx: &mut mpsc::Receiver<String>,
) -> Result<()> {
    use tokio::time::{Duration, interval};

    // Create an interval for UI refreshes
    let mut ui_refresh = interval(Duration::from_millis(100));

    // Track pending message to send
    let mut pending_message: Option<String> = None;

    loop {
        // Draw UI
        terminal
            .draw(|f| ui(f, app))
            .map_err(|e| anyhow::anyhow!("Failed to draw UI: {}", e))?;

        // Use select! to handle all events concurrently
        tokio::select! {
            // Receive agent responses (highest priority)
            Some(response) = rx.recv() => {
                if response.starts_with("Error:") {
                    app.add_error(response);
                } else {
                    app.add_assistant_message(response);
                }
            }

            // Send pending message (only if we have one)
            result = async {
                if let Some(msg) = pending_message.take() {
                    tx.send(msg).await
                } else {
                    futures::future::pending().await
                }
            } => {
                result?;
            }

            // Handle keyboard input
            event_available = tokio::task::spawn_blocking(|| {
                event::poll(Duration::from_millis(50))
            }) => {
                if let Ok(Ok(true)) = event_available
                    && let Ok(Event::Key(key)) = event::read()
                    && key.kind == KeyEventKind::Press
                    && let Some(message) = app.handle_input(key.code)
                {
                    // Queue message for sending
                    pending_message = Some(message);
                }
            }

            // UI refresh interval
            _ = ui_refresh.tick() => {
                // Just triggers redraw in next iteration
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Render the UI
fn ui(frame: &mut Frame, app: &PlanningApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Chat messages
            Constraint::Length(3), // Input box
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    // Render title
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "MPCA Interactive Planning - ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &app.feature_name,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ])])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Render content based on view mode
    match app.view_mode {
        ViewMode::Chat => {
            render_chat_view(frame, app, chunks[1]);
            render_input_box(frame, app, chunks[2]);
        }
        ViewMode::Help => {
            render_help_view(frame, chunks[1]);
        }
    }

    // Render status bar
    let status_color = if app.waiting_for_response {
        Color::Yellow
    } else {
        Color::Green
    };
    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(status_color))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(status, chunks[3]);
}

/// Render the chat message view
fn render_chat_view(frame: &mut Frame, app: &PlanningApp, area: ratatui::layout::Rect) {
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|msg| {
            let (prefix, style) = match msg.role.as_str() {
                "user" => ("You: ", Style::default().fg(Color::Cyan)),
                "assistant" => ("Claude: ", Style::default().fg(Color::Green)),
                "system" => ("System: ", Style::default().fg(Color::Yellow)),
                "error" => ("Error: ", Style::default().fg(Color::Red)),
                _ => ("Unknown: ", Style::default()),
            };

            let content = format!("{}{}", prefix, msg.content);
            ListItem::new(Text::from(content)).style(style)
        })
        .collect();

    let chat_list =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Conversation"));

    frame.render_widget(chat_list, area);
}

/// Render the input box
fn render_input_box(frame: &mut Frame, app: &PlanningApp, area: ratatui::layout::Rect) {
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Type your message (Enter to send)"),
        );
    frame.render_widget(input, area);
}

/// Render the help view
fn render_help_view(frame: &mut Frame, area: ratatui::layout::Rect) {
    let help_text = Text::from(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  q          - Quit the TUI"),
        Line::from("  h          - Show this help screen"),
        Line::from("  Esc        - Return to chat view"),
        Line::from("  Enter      - Send message"),
        Line::from("  Backspace  - Delete character"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "About:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("This TUI enables interactive feature planning with Claude."),
        Line::from(""),
        Line::from("Chat with Claude to refine your feature specifications:"),
        Line::from("  - Feature overview and goals"),
        Line::from("  - Requirements (functional and non-functional)"),
        Line::from("  - Technical design"),
        Line::from("  - Implementation plan"),
        Line::from("  - Testing strategy"),
        Line::from(""),
        Line::from("The conversation will help generate comprehensive"),
        Line::from("specification documents for your feature."),
    ]);

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, area);
}
