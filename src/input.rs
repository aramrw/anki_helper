use crate::app::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.input.char_index.saturating_sub(1);
        self.input.char_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.input.char_index.saturating_add(1);
        self.input.char_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.text.insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn byte_index(&self) -> usize {
        self.input
            .text
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.input.char_index)
            .unwrap_or(self.input.text.len())
    }

