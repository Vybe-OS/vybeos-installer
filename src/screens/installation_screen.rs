use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::{Frame, layout::Rect};

use crate::core::{
    config::Config,
    installer::Installer,
    screen::{Action, Screen},
};

enum InstallEvent {
    Log(String),
    Progress(u16),
    Done,
    Failed(String),
}

enum State {
    Running,
    Done,
    Failed(String),
}

pub struct InstallationScreen {
    rx: Receiver<InstallEvent>,
    logs: Vec<String>,
    progress: u16,
    state: State,
}

impl InstallationScreen {
    pub fn new(installer: Installer) -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut log = |l: &str| {
                let _ = tx.send(InstallEvent::Log(l.to_string()));
            };
            let progress_tx = tx.clone();
            let mut progress = |value| {
                let _ = progress_tx.send(InstallEvent::Progress(value));
            };
            let ev = match installer.run(&mut log, &mut progress) {
                Ok(()) => InstallEvent::Done,
                Err(e) => InstallEvent::Failed(e),
            };
            let _ = tx.send(ev);
        });
        Self { rx, logs: vec![], state: State::Running, progress: 0 }
    }
}

impl Screen for InstallationScreen {
    fn render(&mut self, frame: &mut Frame, area: Rect, _config: &Config) {
        let [text_area, log_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .areas(area);

        let title = match &self.state {
            State::Running => "Installing VybeOS ...".to_string(),
            State::Done => "Installation finished!".to_string(),
            State::Failed(e) => format!("Installation failed: {e}"),
        };
        let text = Paragraph::new(title);
            frame.render_widget(text, text_area);

        let [_, log_area, gauge_area] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(12),
            Constraint::Length(3),
        ])
        .areas(log_area);

        let h = log_area.height.saturating_sub(2) as usize;
        let start = self.logs.len().saturating_sub(h);
        frame.render_widget(
            Paragraph::new(self.logs[start..].join("\n"))
                .block(Block::default().borders(Borders::ALL)),
            log_area,
        );

        frame.render_widget(
            Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Progress"))
                .gauge_style(Style::default().fg(Color::Cyan))
                .label(format!("{}%", self.progress))
                .ratio(f64::from(self.progress) / 100.0),
            gauge_area,
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _config: &mut Config) -> Action {
        match (&self.state, key.code) {
            (State::Running, _) => Action::None,
            (State::Done, KeyCode::Enter) => {
                if std::env::var("VYBEOS_DRY_RUN").is_err() {
                    let _ = Command::new("systemctl").arg("reboot").status();
                }
                Action::Quit
            }
            (State::Failed(_), KeyCode::Esc) => Action::Quit,
            _ => Action::None,
        }
    }

    fn footer_hints(&self) -> &str {
        match self.state {
            State::Running => "Please wait, do not turn off the computer",
            State::Done => "[ENTER] Reboot",
            State::Failed(_) => "[ESC] Quit",
        }
    }

    fn tick(&mut self) -> Action {
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                InstallEvent::Log(l) => self.logs.push(l),
                InstallEvent::Done => self.state = State::Done,
                InstallEvent::Progress(p) => self.progress = p,
                InstallEvent::Failed(e) => self.state = State::Failed(e),
            }
        }
        Action::None
    }
}