use crate::app::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

impl AppState {
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

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.input.char_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.input.char_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.text.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.text.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input.text = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.text.chars().count())
    }

    pub fn reset_input(&mut self) {
        self.input.char_index = 0;
        self.input.text.clear();
    }

    pub fn confirm_search_query(&mut self) {
        let user_input = self.input.text.trim().to_lowercase();
        if let Ok(parsed) = user_input.parse::<usize>() {
            if parsed > 5000 {
                self.input.mode = InputMode::FindID;
                self.select_mode = SelectMode::Expressions;
                return;
            }

            let mut found = false;
            for (i, _) in self.expressions.iter().enumerate() {
                if parsed == i {
                    self.select_mode = SelectMode::Expressions;
                    self.expressions_state.select(Some(i));
                    self.reset_input();
                    found = true;
                    break;
                }
            }
            if !found {
                // for input search, add logic to display fetched results
                self.input.mode = InputMode::Search;
            }
        } else {
            for (i, exp) in self.expressions.iter().enumerate() {
                let dict_word = exp.dict_word.trim().to_lowercase().replace('\n', "");
                if dict_word == user_input {
                    self.select_mode = SelectMode::Expressions;
                    self.expressions_state.select(Some(i));
                    self.reset_input();
                    return;
                }
            }

            self.input.mode = InputMode::Search;
        }
    }

    pub fn rend_input_box(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::from(self.input.text.clone()).style(Color::White))
            .block(
                Block::bordered()
                    .title("Search")
                    // .style(match self.input.mode {
                    //     InputMode::Search => Style::default().light_magenta(),
                    //     InputMode::Grep => Style::default().light_cyan(),
                    //     InputMode::FindID => Style::default().light_blue(),
                    //     _ => Style::default(),
                    // })
                    .style(match self.select_mode {
                        SelectMode::Input => Style::default().light_yellow(),
                        _ => Style::default(),
                    }),
            )
            .render(area, buf);
    }
}
