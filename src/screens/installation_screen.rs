use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect, widgets::{Block, Paragraph}};
use crate::core::{config::Config, installer::Installer, screen::{Screen, Action}};

enum InstallEvent {
    Log(String),
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
    state: State,
}

impl InstallationScreen {
    pub fn new(installer: Installer) -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut log = |l: &str| {
                let _ = tx.send(InstallEvent::Log(l.to_string()));
            };
            let ev = match installer.run(&mut log) {
                Ok(()) => InstallEvent::Done,
                Err(e) => InstallEvent::Failed(e),
            };
            let _ = tx.send(ev);
        });
        Self { rx, logs: vec![], state: State::Running }
    }
}

impl Screen for InstallationScreen {
    fn render(&mut self, frame: &mut Frame, area: Rect, _config: &Config) {
        let title = match &self.state {
            State::Running => "Installing VybeOS...".to_string(),
            State::Done => "Instalasi selesai".to_string(),
            State::Failed(e) => format!("Instalasi gagal: {e}"),
        };
        let h = area.height.saturating_sub(2) as usize;
        let start = self.logs.len().saturating_sub(h);
        frame.render_widget(
            Paragraph::new(self.logs[start..].join("\n"))
                .block(Block::bordered().title(title)),
            area,
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
            (State::Failed(_), KeyCode::Char('q')) => Action::Quit,
            _ => Action::None,
        }
    }

    fn footer_hints(&self) -> &str {
        match self.state {
            State::Running => "Mohon tunggu, jangan matikan komputer",
            State::Done => "Enter: reboot",
            State::Failed(_) => "q: keluar",
        }
    }

    fn tick(&mut self) -> Action {
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                InstallEvent::Log(l) => self.logs.push(l),
                InstallEvent::Done => self.state = State::Done,
                InstallEvent::Failed(e) => self.state = State::Failed(e),
            }
        }
        Action::None
    }
}