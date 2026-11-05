use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame
};
use crate::{config::Config, screen::{Action, Screen}, screens::{welcome_screen::WelcomeScreen}};

// 2. Global state for the installer
pub struct App {
    pub screen: Box<dyn Screen>,
    pub config: Config,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Box::new(WelcomeScreen),
            config: Config::default(),
        }
    }

    pub fn apply(&mut self, action: Action) -> bool {
        match action {
            Action::None => {},
            Action::Goto(next) => self.screen = next,
            Action::Quit => return true,
        }

        false
    }

    pub fn render_ui(frame: &mut Frame, app: &App) {
        let [header_area, body_area, footer_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(frame.area());

        let header = Paragraph::new(" VybeOS Installer ")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, header_area);

        app.screen.render(frame, body_area, &app.config);

        let footer = Paragraph::new(app.screen.footer_hints())
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, footer_area);
    }
}