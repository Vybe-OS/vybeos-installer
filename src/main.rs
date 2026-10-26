use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io::{stdout, Result};

// 1. Define all steps/screens in the installer
#[derive(Debug, PartialEq, Clone)]
enum Screen {
    Welcome,
    SelectDisk,
    Installing { progress: u16 },
    Complete,
}

// 2. Global state for the installer
struct InstallerApp {
    current_screen: Screen,
    selected_disk_index: usize,
    disks: Vec<String>,
}

impl InstallerApp {
    fn new() -> Self {
        Self {
            current_screen: Screen::Welcome,
            selected_disk_index: 0,
            disks: vec![
                String::from("/dev/sda - 512 GB SSD"),
                String::from("/dev/nvme0n1 - 1 TB NVMe"),
            ],
        }
    }

    fn next_screen(&mut self) {
        match &self.current_screen {
            Screen::Welcome => self.current_screen = Screen::SelectDisk,
            Screen::SelectDisk => self.current_screen = Screen::Installing { progress: 0 },
            Screen::Installing { .. } => self.current_screen = Screen::Complete,
            Screen::Complete => {}
        }
    }
}

fn main() -> Result<()> {
    // Setup terminal
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app = InstallerApp::new();

    loop {
        // Render UI based on current state
        terminal.draw(|frame| render_ui(frame, &app))?;

        // Handle async simulated installation progress
        if let Screen::Installing { ref mut progress } = app.current_screen {
            if *progress < 100 {
                *progress += 1;
                std::thread::sleep(std::time::Duration::from_millis(30));
            } else {
                app.next_screen(); // Automatically advance when progress hits 100%
            }
        }

        // Poll keyboard input
        if event::poll(std::time::Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Enter => {
                            if app.current_screen != (Screen::Installing { progress: 0 }) {
                                app.next_screen();
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if app.current_screen == Screen::SelectDisk
                                && app.selected_disk_index < app.disks.len() - 1
                            {
                                app.selected_disk_index += 1;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.current_screen == Screen::SelectDisk
                                && app.selected_disk_index > 0
                            {
                                app.selected_disk_index -= 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Teardown
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn render_ui(frame: &mut Frame, app: &InstallerApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Screen Body
            Constraint::Length(3), // Navigation Bar
        ])
        .split(frame.area());

    // Render global header
    let header = Paragraph::new(" Custom Linux Distro Installer ")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // ROUTER: Render specific screen content using pattern matching
    match &app.current_screen {
        Screen::Welcome => render_welcome_screen(frame, chunks[1]),
        Screen::SelectDisk => render_disk_screen(frame, app, chunks[1]),
        Screen::Installing { progress } => render_install_screen(frame, *progress, chunks[1]),
        Screen::Complete => render_complete_screen(frame, chunks[1]),
    }

    // Render global navigation footer
    let footer_text = match app.current_screen {
        Screen::Welcome => "[Enter] Start Installation | [q] Quit",
        Screen::SelectDisk => "[↑/↓ or k/j] Choose Disk | [Enter] Confirm & Install",
        Screen::Installing { .. } => "Installing OS... Please wait",
        Screen::Complete => "[Enter or q] Exit Installer",
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

// ----------------------------------------------------
// Screen Render Functions
// ----------------------------------------------------

fn render_welcome_screen(frame: &mut Frame, area: Rect) {
    let welcome_text = "\nWelcome to the Linux Installer!\n\nThis wizard will guide you through partitioning, package selection, and installation.\n\nPress [ENTER] to begin.";
    let widget = Paragraph::new(welcome_text)
        .alignment(Alignment::Center)
        .block(Block::default().title(" Step 1: Welcome ").borders(Borders::ALL));
    frame.render_widget(widget, area);
}

fn render_disk_screen(frame: &mut Frame, app: &InstallerApp, area: Rect) {
    let items: Vec<ListItem> = app
        .disks
        .iter()
        .enumerate()
        .map(|(i, disk)| {
            let style = if i == app.selected_disk_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == app.selected_disk_index { "-> " } else { "   " };
            ListItem::new(format!("{prefix}{disk}")).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Step 2: Select Target Drive ")
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn render_install_screen(frame: &mut Frame, progress: u16, area: Rect) {
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Step 3: Extracting Root Filesystem ")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(progress);

    frame.render_widget(gauge, area);
}

fn render_complete_screen(frame: &mut Frame, area: Rect) {
    let text = "\nInstallation Complete!\n\nYour new system is ready. Remove your USB drive and reboot.\n\nPress [ENTER] or [q] to quit.";
    let widget = Paragraph::new(text)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().title(" Finished ").borders(Borders::ALL));
    frame.render_widget(widget, area);
}