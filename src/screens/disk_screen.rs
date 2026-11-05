use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame
};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

use crate::{config::Config, screen::{Action, Screen}};

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Disk {
    pub name: String,
    pub size: String,
    pub model: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DiskScreen {
    pub disks: Vec<Disk>,
    pub selected: usize,
}

#[derive(Deserialize)]
struct LsblkOutput {
    blockdevices: Vec<Disk>,
}

impl DiskScreen {
    pub fn new(disks: Vec<Disk>) -> Self {
        Self {
            disks,
            selected: 0,
        }
    }
    
    pub fn list_disks() -> Result<Vec<Disk>> {
        let output = Command::new("lsblk")
            .args(["-J", "-d", "-o", "NAME,SIZE,MODEL", "-e", "7,11"])
            .output()
            .context("Failed to retrieve: lsblk")?;
    
        if !output.status.success() {
            anyhow::bail!("lsblk exit with status {}", output.status);
        }
    
        let parsed: LsblkOutput = serde_json::from_slice(&output.stdout)
            .context("Failed to parse lsblk")?;
        Ok(parsed.blockdevices)
    }
}

impl Screen for DiskScreen {
    fn render(&self, frame: &mut Frame, area: Rect, _config: &Config) {
        let items: Vec<ListItem> = self
            .disks
            .iter()
            .enumerate()
            .map(|(i, disk)| {
                let style = if i == self.selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if i == self.selected { "-> " } else { "   " };
                ListItem::new(format!("{prefix}{} - {} ({})", disk.name, disk.size, disk.model.as_deref().unwrap_or("-"))).style(style)
            })
            .collect();
    
        let list = List::new(items).block(
            Block::default()
                .title(" Step 2: Select Target Drive ")
                .borders(Borders::ALL),
        );
        frame.render_widget(list, area);
    }

    fn handle_key(&mut self, key: KeyEvent, _config: &mut Config) -> Action {
        match key.code {
            KeyCode::Up if self.selected > 0 => {
                self.selected -= 1;
                Action::None
            }
            KeyCode::Down if self.selected < self.disks.len() - 1 => {
                self.selected += 1;
                Action::None
            }
            KeyCode::Enter => {
                // Here you would typically proceed to the next screen, e.g., partitioning or installation.
                // For now, we just quit for demonstration purposes.
                Action::Quit
            }
            KeyCode::Char('q') => Action::Quit,
            _ => Action::None,
        }
    }

    fn footer_hints(&self) -> &str {
        "Use [UP]/[DOWN] to navigate, [ENTER] to select, [q] to quit"
    }
}
