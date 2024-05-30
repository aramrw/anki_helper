use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, /* Padding */ Paragraph},
};

use crate::app::{AppState, SelectMode};

impl Widget for &mut AppState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ]);
        let [help_area, main_area, info_area] = layout.areas(area);

        self.rend_help_area(help_area, buf);
        self.rend_main(main_area, buf);
        self.rend_info_area(info_area, buf)
    }
}

impl AppState {
    fn rend_info_area(&self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);

        let [msg_area, found_area, _other] = horizontal.areas(area);

        let (msg, style) = match &self.info.msg {
            Some(msg) => (msg.clone(), Style::default()),
            None => (
                format!("Words: [{}]", self.expressions.len()),
                Style::default(),
            ),
        };

        let text = Text::from(Line::from(msg).patch_style(style));
        Paragraph::new(text)
            .block(Block::bordered().title("Information"))
            .render(msg_area, buf);
    }

    fn rend_help_area(&self, area: Rect, buf: &mut Buffer) {
        let horizontal =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);
        let [left, right] = horizontal.areas(area);

        self.rend_keybinds(left, buf);
        self.rend_err(right, buf);
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

        let text = Text::from(Line::from(msg).patch_style(style));
        Paragraph::new(text)
            .block(Block::bordered().title("Keybinds"))
            .render(area, buf);
    }

    fn rend_err(&self, area: Rect, buf: &mut Buffer) {
        let (msg, style) = match &self.err_msg {
            Some(msg) => (msg.clone(), Style::default().light_red().bold()),
            None => (
                "No Errors :)".to_string(),
                Style::default().light_green().bold(),
            ),
        };

        let text = Text::from(Line::from(msg).patch_style(style));
        Paragraph::new(text)
            .block(Block::bordered().title("Errors"))
            .render(area, buf);
    }

    fn rend_main(&mut self, area: Rect, buf: &mut Buffer) {
        let horizontal =
            Layout::horizontal([Constraint::Percentage(15), Constraint::Percentage(85)]);
        let [expressions_area, sentences_area] = horizontal.areas(area);

        {
            let words: Vec<ListItem> = self
                .expressions
                .iter()
                .enumerate()
                .map(|(i, word)| word.to_list_item(i))
                .collect();

            let words = List::new(words)
                .block(
                    Block::bordered()
                        .title("Expressions")
                        .style(match self.select_mode {
                            SelectMode::Expressions => Style::default().yellow().bold(),
                            SelectMode::Sentences => Style::default(),
                        }),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED)
                        .fg(Color::White),
                )
                .highlight_symbol("⇢ ")
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

            StatefulWidget::render(words, expressions_area, buf, &mut self.expressions_state);
        }

        let mut sentence_items: Vec<ListItem> = Vec::new();

        if let Some(i) = self.selected_expression {
            let sentences = &self.expressions[i].sentences.clone();
            if let Some(sentences) = sentences {
                sentence_items = sentences
                    .iter()
                    .enumerate()
                    .map(|(i, sentence)| sentence.to_list_item(i))
                    .collect();
            };

            let sentences = List::new(sentence_items)
                .block(
                    Block::bordered()
                        .title(format!(
                            "{}'s Sentences",
                            &self.expressions[i].dict_word.clone()
                        ))
                        .style(match self.select_mode {
                            SelectMode::Expressions => Style::default(),
                            SelectMode::Sentences => Style::default().yellow().bold(),
                        }),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED)
                        .fg(Color::White),
                )
                .highlight_symbol("⇢ ")
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

            StatefulWidget::render(
                sentences,
                sentences_area,
                buf,
                &mut self.expressions[i].sentences_state,
            );
        }
    }
}
