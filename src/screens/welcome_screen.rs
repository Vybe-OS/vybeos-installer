use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph},
    crossterm::event::{KeyCode, KeyEvent},
    Frame
};

use crate::{config::Config, screen::{Action, Screen}, screens::disk_screen::DiskScreen};
pub struct WelcomeScreen;

impl Screen for WelcomeScreen {
    fn render(&self, frame: &mut Frame, area: Rect, _config: &Config) {
        let welcome_text = "\nWelcome to the VybeOS Installer!\n\nThis wizard will guide you through partitioning, package selection, and installation.\n\nPress [ENTER] to begin.";
        let widget = Paragraph::new(welcome_text)
            .alignment(Alignment::Center)
            .block(Block::default().title(" Step 1: Welcome to VybeOS ").borders(Borders::ALL));
        frame.render_widget(widget, area);
    }

    fn handle_key(&mut self, key: KeyEvent, _config: &mut Config) -> Action {
        let disk = DiskScreen::list_disks().unwrap_or_default();
        match key.code {
            KeyCode::Enter => Action::Goto(Box::new(DiskScreen::new(disk))),
            KeyCode::Char('q') => Action::Quit,
            _ => Action::None,
        }
    }

    fn footer_hints(&self) -> &str {
        "Press [ENTER] to continue, [q] to quit"
    }
}