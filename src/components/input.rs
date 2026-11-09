// input.rs
use ratatui::{
    buffer::Buffer, crossterm::event::{KeyCode, KeyEvent, KeyModifiers}, layout::{Position, Rect}, style::{Color, Style}, widgets::{Block, Padding, Paragraph, StatefulWidget, Widget},
};

#[derive(Default)]
pub struct InputState {
    value: String,
    cursor: usize,
    pub cursor_pos: Option<Position>,
}

impl InputState {
    pub fn value(&self) -> &str {
        &self.value
    }

    fn byte_index(&self) -> usize {
        self.value
            .char_indices()
            .nth(self.cursor)
            .map_or(self.value.len(), |(i, _)| i)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        let len = self.value.chars().count();
        match key.code {
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let i = self.byte_index();
                self.value.insert(i, c);
                self.cursor += 1;
            }
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                let i = self.byte_index();
                self.value.remove(i);
            }
            KeyCode::Delete if self.cursor < len => {
                let i = self.byte_index();
                self.value.remove(i);
            }
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(len),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = len,
            _ => return false,
        }
        true
    }
}

pub struct Input<'a> {
    title: &'a str,
    masked: bool,
    focused: bool,
}

impl<'a> Input<'a> {
    pub fn new(title: &'a str) -> Self {
        Self { title, masked: false, focused: false }
    }
    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl StatefulWidget for Input<'_> {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut InputState) {
        let border = if self.focused { Color::Cyan } else { Color::DarkGray };
        let block = Block::bordered()
            .title(self.title)
            .border_style(Style::new().fg(border))
            .padding(Padding::new(1, 1, 0, 0));
        let inner = block.inner(area);
        
        let text = if self.masked {
            "•".repeat(state.value.chars().count())
        } else {
            state.value.clone()
        };
        Paragraph::new(text).block(block).render(area, buf);

        state.cursor_pos = self
            .focused
            .then(|| Position::new(inner.x + state.cursor as u16, inner.y));
    }
}