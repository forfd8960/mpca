//! Terminal User Interface for interactive planning mode.
//!
//! Provides a ratatui-based TUI for interactively planning features with Claude.
//! Users can review generated specs, edit content, and approve or regenerate plans.

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use mpca_core::AgentRuntime;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io;

/// Application state for the planning TUI
struct PlanningApp {
    /// Feature slug being planned
    feature_name: String,

    /// Current spec content (will be populated by agent)
    spec_content: String,

    /// Current view mode
    view_mode: ViewMode,

    /// Status message
    status: String,

    /// Whether the app should exit
    should_quit: bool,
}

/// View modes for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    /// Viewing the feature spec
    Spec,

    /// Help screen
    Help,
}

impl PlanningApp {
    /// Creates a new planning app
    fn new(feature_name: String) -> Self {
        Self {
            feature_name,
            spec_content: "Planning in progress...\n\nThis feature will be implemented in Stage 5."
                .to_string(),
            view_mode: ViewMode::Spec,
            status: "Press 'q' to quit, 'h' for help".to_string(),
            should_quit: false,
        }
    }

    /// Handle keyboard input
    fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('h') => {
                self.view_mode = ViewMode::Help;
                self.status = "Viewing help - press 'q' to go back".to_string();
            }
            KeyCode::Esc => {
                if self.view_mode == ViewMode::Help {
                    self.view_mode = ViewMode::Spec;
                    self.status = "Press 'q' to quit, 'h' for help".to_string();
                }
            }
            _ => {}
        }
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

    // Run the event loop
    let result = run_app(&mut terminal, &mut app).await;

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

/// Run the main application loop
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut PlanningApp,
) -> Result<()> {
    loop {
        terminal
            .draw(|f| ui(f, app))
            .map_err(|e| anyhow::anyhow!("Failed to draw UI: {}", e))?;

        // Poll for events with timeout
        if event::poll(std::time::Duration::from_millis(100)).context("Failed to poll events")?
            && let Event::Key(key) = event::read().context("Failed to read event")?
        {
            // Only process key press events (not release)
            if key.kind == KeyEventKind::Press {
                app.handle_input(key.code);
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
            Constraint::Min(10),   // Content
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
        ViewMode::Spec => render_spec_view(frame, app, chunks[1]),
        ViewMode::Help => render_help_view(frame, chunks[1]),
    }

    // Render status bar
    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(status, chunks[2]);
}

/// Render the spec view
fn render_spec_view(frame: &mut Frame, app: &PlanningApp, area: ratatui::layout::Rect) {
    let spec = Paragraph::new(app.spec_content.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Feature Specification"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(spec, area);
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
        Line::from("  Esc        - Return to spec view"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "About:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("This TUI enables interactive feature planning with Claude."),
        Line::from("Full functionality will be implemented in Stage 5."),
        Line::from(""),
        Line::from("Features coming soon:"),
        Line::from("  - Live chat with Claude"),
        Line::from("  - Spec editing and refinement"),
        Line::from("  - Plan approval and regeneration"),
        Line::from("  - Git worktree creation"),
    ]);

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, area);
}
