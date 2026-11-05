use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use crate::config::Config;

pub enum Action {
    None,
    Goto(Box<dyn Screen>),
    Quit,
}

pub trait Screen {
    fn render(&self, frame: &mut Frame, area: Rect, config: &Config);
    fn handle_key(&mut self, key: KeyEvent, config: &mut Config) -> Action;
    fn footer_hints(&self) -> &str;
    fn tick(&mut self) -> Action {
        Action::None
    }
}