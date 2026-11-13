use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame, layout::{Constraint, Layout, Rect}, style::{Color, Modifier, Style}, widgets::{Block, Borders, Paragraph, Row, Table, TableState}
};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{path::Path, process::Command};

use crate::{core::config::Config, core::screen::{Action, Screen}, screens::system_config_screen::SystemConfigScreen};

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Disk {
    pub name: String,
    pub size: String,
    pub model: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DiskScreen {
    pub disks: Vec<Disk>,
    pub table_state: TableState
}

#[derive(Deserialize)]
struct LsblkOutput {
    blockdevices: Vec<Disk>,
}

impl DiskScreen {
    pub fn new(disks: Vec<Disk>) -> Self {
        let mut table_state = TableState::default();
        if !disks.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            disks,
            table_state,
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
        Ok(
            parsed
            .blockdevices
            .into_iter()
            .filter(
                |d| Path::new("/sys/block")
                    .join(&d.name)
                    .join("device")
                    .exists()
            )
            .collect()
        )
    }
}

impl Screen for DiskScreen {
    fn render(&mut self, frame: &mut Frame, area: Rect, _config: &Config) {
        let table_height = self.disks.len() as u16 + 2 + 2;
        let [text_area, table_area, footer_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(table_height),
            Constraint::Min(0),
        ])
        .areas(area);

        let text = Paragraph::new("DISK PARTITIONING\nSelect a disk to partition and install VybeOS.");
        frame.render_widget(text, text_area);

        let header = Row::new(["Device", "Size", "Model"])
            .style(Style::default().add_modifier(Modifier::BOLD)).bottom_margin(1);

        let rows: Vec<Row> = self.disks.iter().map(|disk| {
            Row::new([disk.name.clone(), disk.size.clone(), disk.model.as_deref().unwrap_or("-").to_string()])
        }).collect();

        let widths = [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(1)
            .row_highlight_style(Style::default().fg(Color::Cyan))
            .highlight_symbol("-> ")
            .block(Block::default().borders(Borders::ALL));
        frame.render_stateful_widget(table, table_area, &mut self.table_state);

        let [_, footer_area] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(footer_area);

        let footer = Paragraph::new("[WARNING] All data in the selected disk will be lost.")
            .style(Style::default().fg(Color::Yellow));

        frame.render_widget(footer, footer_area);
    }

    fn handle_key(&mut self, key: KeyEvent, config: &mut Config) -> Action {
        match key.code {
            KeyCode::Up => {
                self.table_state.select_previous();
                Action::None
            }
            KeyCode::Down => {
                self.table_state.select_next();
                Action::None
            }
            KeyCode::Enter => {
                if let Some(i) = self.table_state.selected() {
                    config.disk = self.disks.get(i).map(|d| d.name.clone());
                    return Action::Goto(Box::new(SystemConfigScreen::new()));
                } else {
                    Action::None
                }
            }
            KeyCode::Esc => Action::Quit,
            _ => Action::None,
        }
    }

    fn footer_hints(&self) -> &str {
        "[↑/↓] Navigate    [ENTER] Select    [ESC] Quit"
    }
}
