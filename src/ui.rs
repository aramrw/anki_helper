use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph},
    widgets::{Block, List, ListItem, Padding, Paragraph},
};

use crate::app::AppState;
use crate::app::{AppState, SelectMode};

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
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ]);
        let [help_area, main_area, info_area] = layout.areas(area);

        self.rend_help_area(help_area, buf);
        self.rend_main(main_area, buf);
    }
}

    fn rend_keybinds(&self, area: Rect, buf: &mut Buffer) {
        let (msg, style) = match self.select_mode {
            SelectMode::Expressions => (
                vec![
                    "<Enter> ".yellow().bold(),
                    "Sentence Selection ".into(),
                    "<Up> ".yellow().bold(),
                    "Select Prev ".into(),
                    "<Down> ".yellow().bold(),
                    "Select Next".into(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            SelectMode::Sentences => (
                vec![
                    "<Esc> ".yellow().bold(),
                    "Word Selection ".into(),
                    "<Up> ".yellow().bold(),
                    "Select Prev ".into(),
                    "<Down> ".yellow().bold(),
                    "Select Next".into(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
        };

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
