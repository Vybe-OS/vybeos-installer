mod app;
mod config;
mod screen;
mod screens;

use crossterm::{
    event::{self, Event, KeyEventKind},
};
use ratatui::{
    DefaultTerminal
};
use anyhow::{Ok, Result};
use crate::app::App;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| App::render_ui(frame, &mut app))?;

        if event::poll(std::time::Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let action = app.screen.handle_key(key, &mut app.config);
                    if app.apply(action) {
                        return Ok(());
                    }
                }
            }
        }

        let action = app.screen.tick();
        if app.apply(action) {
            return Ok(());
        }
    }
}