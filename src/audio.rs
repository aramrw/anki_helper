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

    fn render_top_bar(&self, area: Rect, buf: &mut Buffer) {
        if let Some(sentence) = self.get_current_sentence() {
            let media_title = sentence.media_title;
            let horizontal = Layout::horizontal([
                Constraint::Length((media_title.len() + 2) as u16),
                Constraint::Min(5),
                Constraint::Length(17),
                Constraint::Min(15),
            ]);
            let [media_title_area, sentence_area, play_button, keybinds_area] =
                horizontal.areas(area);

            Paragraph::new(Span::from(media_title).style(Color::White))
                .block(Block::bordered().title("Media Title"))
                .fg(Color::Yellow)
                .render(media_title_area, buf);

            Paragraph::new(Span::from(sentence.sentence).style(Color::White))
                .block(Block::bordered().title("Sentence"))
                .fg(Color::Yellow)
                .render(sentence_area, buf);

            // play btn
            let (mut msg, mut style) = (vec!["<P> ".blue(), "Play Audio".into()], Style::default());
            let mut text = Text::from(Line::from(msg)).patch_style(style);
            Paragraph::new(text)
                .block(Block::bordered())
                .fg(Color::LightBlue)
                .render(play_button, buf);

            (msg, style) = (
                vec![
                    "<Left> ".blue(),
                    "Trim Less ".white(),
                    "<Right> ".blue(),
                    "Trim More ".white(),
                    "<R> ".light_red(),
                    "Reset ".white(),
                    "<C-Enter> ".light_green(),
                    "Confirm".white(),
                ],
                Style::default(),
            );
            text = Text::from(Line::from(msg)).patch_style(style);
            Paragraph::new(text)
                .block(Block::bordered())
                .centered()
                .fg(Color::LightCyan)
                .render(keybinds_area, buf);
        }
    }
}
