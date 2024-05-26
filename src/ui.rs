use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph},
};

use crate::app::AppState;

impl Widget for &mut AppState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Min(3)]);
        let [main] = layout.areas(area);

        // Delete This Paragraph!
        Paragraph::new(
            "DELETE @ -> src\\app.rs -> fn render() {\n// Delete This Paragraph!    
            .block(Block::bordered().title(\"Delete Me!\"))
            .fg(Color::Red)
            .render(main, buf)
        };
            ",
        )
        .block(
            Block::bordered()
                .title("Delete Me!")
                .padding(Padding::new(1, 1, 1, 1)),
        )
        .fg(Color::Red)
        .render(main, buf);
    }
}
