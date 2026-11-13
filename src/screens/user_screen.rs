use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::{Constraint, Layout, Rect}, style::{Color, Style}, widgets::{Block, Paragraph}};

use crate::{components::input::{Input, InputState}, core::config::Config, core::screen::{Action, Screen}, screens::{summary_screen::SummaryScreen}};

#[derive(Clone, Copy, PartialEq)]
enum FocusField {
    Username,
    Password,
    ConfirmPassword,
}

pub struct UserConfigScreen {
    username: InputState,
    password: InputState,
    confirm_password: InputState,
    focus: FocusField,
    error: Option<String>,
    show_password: bool,
}

impl UserConfigScreen {
    pub fn new() -> Self {
        Self {
            username: Default::default(),
            password: Default::default(),
            confirm_password: Default::default(),
            focus: FocusField::Username,
            error: None,
            show_password: false,
        }
    }

    fn active_input(&mut self) -> &mut InputState {
        match self.focus {
            FocusField::Username => &mut self.username,
            FocusField::Password => &mut self.password,
            FocusField::ConfirmPassword => &mut self.confirm_password,
        }
    }

    fn toggle_show_password(&mut self) {
        self.show_password = !self.show_password;
    }

    fn is_masked(&self, field: FocusField) -> bool {
        field != FocusField::Username && !self.show_password
    }
}

impl FocusField {
    fn next(self) -> Self {
        match self {
            FocusField::Username => FocusField::Password,
            FocusField::Password => FocusField::ConfirmPassword,
            FocusField::ConfirmPassword => FocusField::Username,
        }
    }
}

impl Screen for UserConfigScreen {
    fn render(&mut self, frame: &mut Frame, area: Rect, _config: &Config) {
        let [text_area, username_field_area, password_field_area, error_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(area);

        let text = Paragraph::new("USER CONFIGURATION\nPlease enter the username and password for your system.");
        frame.render_widget(text, text_area);

        frame.render_stateful_widget(Input::new(" Username ").focused(self.focus == FocusField::Username), username_field_area, &mut self.username);

        let password = Block::default();
        let inner_password = password.inner(password_field_area);

        let [password_field_area, password_confirm_field_area] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .areas(inner_password);

        frame.render_stateful_widget(Input::new(" Password ").masked(self.is_masked(FocusField::Password)).focused(self.focus == FocusField::Password), password_field_area, &mut self.password);

        frame.render_stateful_widget(Input::new(" Confirm Password ").masked(self.is_masked(FocusField::ConfirmPassword)).focused(self.focus == FocusField::ConfirmPassword), password_confirm_field_area, &mut self.confirm_password);

        let text = Paragraph::new(self.error.as_deref().unwrap_or_default())
            .style(Style::default().fg(Color::Red));
        frame.render_widget(text, error_area);

        let pos = match self.focus {
            FocusField::Username => self.username.cursor_pos,
            FocusField::Password => self.password.cursor_pos,
            FocusField::ConfirmPassword => self.confirm_password.cursor_pos,
        };
        if let Some(pos) = pos {
            frame.set_cursor_position(pos);
        }
    }

    fn handle_key(&mut self, key: KeyEvent, config: &mut Config) -> Action {
        if self.active_input().handle_key(key) {
            self.error = None;
            return Action::None;
        }

        match key.code {
            KeyCode::Tab | KeyCode::Down => {
                self.focus = self.focus.next();
                Action::None
            },
            KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.toggle_show_password();
                Action::None
            },
            KeyCode::Enter => {
                if self.username.value().is_empty() {
                    self.error = Some("Username is required".into());
                    return Action::None;
                }
                if self.password.value().is_empty() {
                    self.error = Some("Password is required".into());
                    return Action::None;
                }
                if self.password.value() != self.confirm_password.value() {
                    self.error = Some("Passwords do not match".into());
                    return Action::None;
                }

                config.username = Some(self.username.value().to_owned());
                config.password = Some(self.password.value().to_owned());
                Action::Goto(Box::new(SummaryScreen::new()))
            },
            KeyCode::Esc => Action::Quit,
            _ => Action::None,
        }
    }

    fn footer_hints(&self) -> &str {
        "[TAB] Switch    [CTRL+SPACE] Show Password    [ENTER] Next Step    [ESC] Quit"
    }
}