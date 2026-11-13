use std::process::Command;

use anyhow::{Result, Context};
use crossterm::event::KeyCode;
use ratatui::{Frame, layout::{Constraint, Layout, Rect}, style::{Color, Modifier, Style}, widgets::{Block, Borders, Paragraph, Row, Table, TableState}};

use crate::{config::Config, screen::{Action, Screen}, screens::disk_screen::DiskScreen};

#[derive(Debug, Clone, PartialEq)]
enum TableStateFocus {
    Locale,
    Timezone
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTimezoneScreen {
    pub locales: Vec<String>,
    pub timezones: Vec<String>,
    pub locale_state: TableState,
    pub timezone_state: TableState,
    focus: TableStateFocus,
}

const DEFAULT_LOCALE: &str = "id_ID.UTF-8";
const DEFAULT_TIMEZONE: &str = "Asia/Jakarta";

impl LocaleTimezoneScreen {
    pub fn new(locales: Vec<String>, timezones: Vec<String>) -> Self {
        let mut locale_state = TableState::default();
        let mut timezone_state = TableState::default();
        Self::select_default(&mut locale_state, &locales, DEFAULT_LOCALE);
        Self::select_default(&mut timezone_state, &timezones, DEFAULT_TIMEZONE);
        

        Self {
            locales,
            timezones,
            locale_state,
            timezone_state,
            focus: TableStateFocus::Locale,
        }
    }

    pub fn list_locales() -> Result<Vec<String>> {
        let output = std::fs::read_to_string("/usr/share/i18n/SUPPORTED")
            .context("Failed to read /usr/share/i18n/SUPPORTED")?;

        Ok(output
            .lines()
            .filter(|l| l.ends_with("UTF-8"))
            .filter_map(|l| l.split_whitespace().next())
            .map(str::to_owned)
            .collect()
        )
    }

    pub fn list_timezones() -> Result<Vec<String>> {
        let output = Command::new("timedatectl")
            .args(["list-timezones"])
            .output()
            .context("Failed to retrieve: timedatectl list-timezones")?;
    
        if !output.status.success() {
            anyhow::bail!("timedatectl exit with status {}", output.status);
        }
    
        let timezones = String::from_utf8(output.stdout)
            .context("Failed to parse timedatectl list-timezones")?;
        Ok(timezones.lines().map(|s| s.to_string()).collect())
    }

    pub fn active_table_state(&mut self) -> &mut TableState {
        match self.focus {
            TableStateFocus::Locale => &mut self.locale_state,
            TableStateFocus::Timezone => &mut self.timezone_state,
        }
    }

    fn select_default(state: &mut TableState, items: &[String], default: &str) {
        if items.is_empty() {
            return;
        }
        let idx = items.iter().position(|s| s == default).unwrap_or(0);
        state.select(Some(idx));
    }
}

impl Screen for LocaleTimezoneScreen {
    fn render(&mut self, frame: &mut Frame, area: Rect, _config: &crate::config::Config) {
        let [text_area, table_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1)
        ])
        .areas(area);

        let text = Paragraph::new("LOCALE & TIMEZONE\nSelect a locale and timezone for your system.");
            frame.render_widget(text, text_area);

        let table = Block::default();
        let inner_table = table.inner(table_area);

        let [locale_table_area, timezone_table_area] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50)
        ])
        .areas(inner_table);
        let widths = [
            Constraint::Min(1),
        ];

        let locale_header = Row::new(["Locale"])
            .style(Style::default().add_modifier(Modifier::BOLD)).bottom_margin(1);

        let locale_rows: Vec<Row> = self.locales.iter().map(|locale| {
            Row::new([locale.clone(), "".to_string(), "".to_string()])
        }).collect();

        let locale_table = Table::new(locale_rows, widths)
            .header(locale_header)
            .column_spacing(1)
            .style(if self.focus == TableStateFocus::Locale {
                Color::White
            } else {
                Color::DarkGray
            })
            .row_highlight_style(if self.focus == TableStateFocus::Locale {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            })
            .highlight_symbol("-> ")
            .block(Block::default().borders(Borders::ALL));
        frame.render_stateful_widget(locale_table, locale_table_area, &mut self.locale_state);

        let timezone_header = Row::new(["Timezone"])
            .style(Style::default().add_modifier(Modifier::BOLD)).bottom_margin(1);

        let timezone_rows: Vec<Row> = self.timezones.iter().map(|timezone| {
            Row::new([timezone.clone(), "".to_string(), "".to_string()])
        }).collect();

        let timezone_table = Table::new(timezone_rows, widths)
            .header(timezone_header)
            .column_spacing(1)
            .style(if self.focus == TableStateFocus::Timezone {
                Color::White
            } else {
                Color::DarkGray
            })
            .row_highlight_style(if self.focus == TableStateFocus::Timezone {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            })
            .highlight_symbol("-> ")
            .block(Block::default().borders(Borders::ALL));
        frame.render_stateful_widget(timezone_table, timezone_table_area, &mut self.timezone_state);
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent, config: &mut Config) -> Action {
        match key.code {
            KeyCode::Tab | KeyCode::Right | KeyCode::Left => {
                self.focus = match self.focus {
                    TableStateFocus::Locale => TableStateFocus::Timezone,
                    TableStateFocus::Timezone => TableStateFocus::Locale,
                };
                Action::None
            }
            KeyCode::Up => {
                self.active_table_state().select_previous();
                Action::None
            }
            KeyCode::Down => {
                self.active_table_state().select_next();
                Action::None
            }
            KeyCode::Enter => {
                config.locale = self.locale_state.selected().and_then(|i| self.locales.get(i).cloned());
                config.timezone = self.timezone_state.selected().and_then(|i| self.timezones.get(i).cloned());

                let disks = DiskScreen::list_disks().unwrap_or_default();
                return Action::Goto(Box::new(DiskScreen::new(disks)));
            }
            KeyCode::Esc => Action::Quit,
            _ => Action::None,
        }
    }

    fn footer_hints(&self) -> &str {
        "[↑/↓] Navigate    [TAB/←/→] Switch Table    [ENTER] Next Step    [ESC] Quit"
    }
}