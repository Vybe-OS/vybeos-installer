use crossterm::{event::{KeyCode, KeyEvent}};
use ratatui::{Frame, layout::{Constraint, Layout, Rect}, style::{Color, Style}, widgets::Paragraph};

use crate::{components::input::{Input, InputState}, core::config::Config, core::screen::{Action, Screen}, screens::user_screen::UserConfigScreen};

pub struct SystemConfigScreen {
    pub fields: [InputState; 1],
    pub focus: usize,
    pub error: Option<String>
}

impl SystemConfigScreen {
    pub fn new() -> Self {
        Self { 
            fields: Default::default(),
            focus: 0,
            error: None
        }
    }
}

impl Screen for SystemConfigScreen {
    fn render(&mut self, frame: &mut Frame, area: Rect, _config: &Config) {
        let [text_area, hostname_field_area, error_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(area);

        let text = Paragraph::new("SYSTEM CONFIGURATION\nPlease enter the hostname for your system.");
        frame.render_widget(text, text_area);

        frame.render_stateful_widget(Input::new(" Hostname ").focused(true), hostname_field_area, &mut self.fields[self.focus]);
        if let Some(pos) = self.fields[self.focus].cursor_pos {
            frame.set_cursor_position(pos);
        }

        let text = Paragraph::new(self.error.as_deref().unwrap_or_default())
            .style(Style::default().fg(Color::Red));
        frame.render_widget(text, error_area);
    }

    fn handle_key(&mut self, key: KeyEvent, config: &mut Config) -> Action {
        if self.fields[self.focus].handle_key(key) {
            self.error = None;
            return Action::None;
        }

        match key.code {
            KeyCode::Enter => {
                let [hostname] = &self.fields;
                if hostname.value().is_empty() {
                    self.error = Some("Hostname is required".into());
                    return Action::None;
                } else {
                    config.hostname = Some(hostname.value().to_owned());
                    return Action::Goto(Box::new(UserConfigScreen::new()));
                }
            }
            KeyCode::Esc => Action::Quit,
            _ => Action::None,
        }
    }

    fn footer_hints(&self) -> &str {
        "[ENTER] Next Step    [ESC] Quit"
    }
}