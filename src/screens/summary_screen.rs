use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::{Constraint, Layout, Rect}, style::{Color, Style}, widgets::{Block, Borders, Padding, Paragraph}};

use crate::{core::{config::Config, screen::{Action, Screen}}, screens::installation_screen::InstallationScreen};

pub struct SummaryScreen {}

impl SummaryScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for SummaryScreen {
    fn render(&mut self, frame: &mut Frame, area: Rect, config: &Config) {
        let [text_area, summary_area, footer_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .areas(area);

        let text = Paragraph::new("INSTALLATION SUMMARY\nReview your settings before proceeding.");
        frame.render_widget(text, text_area);

        let summary_text = format!(
            "Disk      : {}\nLocale    : {}\nTimezone  : {}\nHostname  : {}\nUsername  : {}\nPassword  : {}",
            config.disk.as_deref().unwrap_or("-"),
            config.locale.as_deref().unwrap_or("-"),
            config.timezone.as_deref().unwrap_or("-"),
            config.hostname.as_deref().unwrap_or("-"),
            config.username.as_deref().unwrap_or("-"),
            config.password.as_deref().unwrap_or("-")
        );
        let summary = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).padding(Padding::new(1, 1, 0, 0)));
        frame.render_widget(summary, summary_area);

        let [_, footer_area] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(footer_area);

        let footer = Paragraph::new(format!("[WARNING] All data on {} will be permanently formatted and overwritten.", config.disk.as_deref().unwrap_or("-")))
            .style(Style::default().fg(Color::Yellow));

        frame.render_widget(footer, footer_area);
    }

    fn handle_key(&mut self, key: KeyEvent, _config: &mut Config) -> Action {
        match key.code {
            KeyCode::Enter => Action::Goto(Box::new(InstallationScreen::new(_config.validate().expect("Invalid configuration")))),
            KeyCode::Esc => Action::Quit,
            _ => Action::None,
        }
    }

    fn footer_hints(&self) -> &str {
        "[ENTER] Next Step    [ESC] Quit"
    }
}