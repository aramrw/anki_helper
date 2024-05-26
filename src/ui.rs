use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph},
};

use crate::app::AppState;

impl Widget for &mut AppState {
    fn render(self, area: Rect, buf: &mut Buffer) {
