use ratatui::{
    Frame, layout::{Alignment, Constraint, Layout}, style::{Color, Style}, widgets::{Block, Borders, List, ListItem, Paragraph}
};
use crate::{config::Config, screen::{Action, Screen}, screens::{welcome_screen::WelcomeScreen}};

pub struct App {
    pub screen: Box<dyn Screen>,
    pub config: Config,
    pub step: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Box::new(WelcomeScreen),
            config: Config::default(),
            step: 1,
        }
    }

    pub fn apply(&mut self, action: Action) -> bool {
        match action {
            Action::None => {},
            Action::Goto(next) => {
                self.screen = next;
                self.step += 1;
            },
            Action::Quit => return true,
        }

        false
    }

    pub fn render_ui(frame: &mut Frame, app: &App) {
        let [body_area, footer_area] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(frame.area());
        
        let body = Block::default()
            .borders(Borders::ALL)
            .title(" [ VybeOS Installer ] ")
            .title_alignment(Alignment::Center);
        let inner_body = body.inner(body_area);
        frame.render_widget(body, body_area);


        let [navigation_area, content_area] = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ])
        .areas(inner_body);

        let steps = vec![
            "Welcome",
            "Partitions",
            "User Setup",
            "System Config",
            "Summary",
            "Installing"
        ];
        let nav_item: Vec<ListItem> = steps
            .iter()
            .enumerate()
            .map(|(i, step)| {
                let style = if i + 1 == app.step {
                    Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)
                } else if i + 1 < app.step {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                    
                };
                let prefix = if i + 1 == app.step { "· " } else { "  " };
                ListItem::new(format!("{}{}", prefix, step)).style(style)
            })
            .collect();
        let navigation = List::new(nav_item)
            .block(Block::default().borders(Borders::RIGHT))
            .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::BOLD));

        frame.render_widget(navigation, navigation_area);
        app.screen.render(frame, content_area, &app.config);

        let footer = Paragraph::new(app.screen.footer_hints())
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, footer_area);
    }
}