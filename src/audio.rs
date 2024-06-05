use crate::app::{AppState, Pages, SelectMode};
use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

    pub fn rend_splice_page(&self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ]);
        let [top_bar, middle_area, bottom_bar] = vertical.areas(area);
        self.render_top_bar(top_bar, buf);
    }

