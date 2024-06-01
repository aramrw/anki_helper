use crate::app::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.input.char_index.saturating_sub(1);
        self.input.char_index = self.clamp_cursor(cursor_moved_left);
    }

