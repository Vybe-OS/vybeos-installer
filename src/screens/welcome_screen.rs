use ratatui::{
    Frame, crossterm::event::{KeyCode, KeyEvent}, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, widgets::{Paragraph, Wrap}
};

use crate::{core::config::Config, core::screen::{Action, Screen}, screens::{locale_timezone_screen::LocaleTimezoneScreen}};
pub struct WelcomeScreen;

impl Screen for WelcomeScreen {
    fn render(&mut self, frame: &mut Frame, area: Rect, _config: &Config) {
        let ascii_logo = include_str!("../assets/logo.txt");
        let logo_lines = ascii_logo.lines().count() as u16;

        let [_spacer, logo_area, text_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Length(logo_lines),
                Constraint::Min(0),
            ])
            .areas(area);

        let logo_widget = Paragraph::new(ascii_logo)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);

        frame.render_widget(logo_widget, logo_area);

        let welcome_text = "\nWelcome to the VybeOS Installer!\nThis wizard will guide you through partitioning, setup, and installation.\n\nPress [ENTER] to begin.";
        let text = Paragraph::new(welcome_text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(text, text_area);
    }

    fn handle_key(&mut self, key: KeyEvent, _config: &mut Config) -> Action {
        let locales = LocaleTimezoneScreen::list_locales().unwrap_or_default();
        let timezones = LocaleTimezoneScreen::list_timezones().unwrap_or_default();
        match key.code {
            KeyCode::Enter => Action::Goto(Box::new(LocaleTimezoneScreen::new(locales, timezones))),
            KeyCode::Esc => Action::Quit,
            _ => Action::None,
        }
    }

    fn footer_hints(&self) -> &str {
        "[ENTER] Next Step    [ESC] Quit"
    }
}