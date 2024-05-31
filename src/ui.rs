use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, /* Padding */ Paragraph},
};

use crate::app::{AppState, SelectMode, Sentence};

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
        let horizontal =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);

        let [msg_area, err_area] = horizontal.areas(area);

        let (msg, style) = match &self.info.msg {
            Some(msg) => (msg.clone(), Style::default().bold().fg(Color::Blue)),
            None => (
                format!("Words: [{}]", self.expressions.len()),
                Style::default(),
            ),
        };

        let text = Text::from(Line::from(msg).patch_style(style));
        Paragraph::new(text)
            .block(Block::bordered().title("Information"))
            .render(msg_area, buf);

        self.rend_err(err_area, buf);
    }

    fn rend_help_area(&mut self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ]);
        let [left, mid_left, mid_right, right] = horizontal.areas(area);

        self.rend_keybinds(right, buf);
    }

    fn rend_keybinds(&self, area: Rect, buf: &mut Buffer) {
        let (msg, style) = match self.select_mode {
            SelectMode::Expressions => (
                vec![
                    "(".into(),
                    "<Up> ".light_yellow().bold(),
                    "Prev ".yellow(),
                    "| ".into(),
                    "<Down> ".light_yellow().bold(),
                    "Next".yellow(),
                    ") ".into(),
                    "<Enter> ".light_green().bold(),
                    "Sentence Selection ".green(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            SelectMode::Sentences => (
                vec![
                    "<Esc> ".light_red().bold(),
                    "Back ".red(),
                    "(".into(),
                    "<Up> ".light_yellow().bold(),
                    "Prev ".yellow(),
                    "| ".into(),
                    "<Down> ".light_yellow().bold(),
                    "Next ".yellow(),
                    ") ".into(),
                    "<P> ".light_blue().bold(),
                    "Play Audio ".blue(),
                    "<C> ".light_green().bold(),
                    "Update Card".green(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
        };

        let text = Text::from(Line::from(msg).patch_style(style));
        Paragraph::new(text)
            .block(Block::bordered().title("Keybinds"))
            .centered()
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

    fn rend_media_title(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(sentence) = &self.get_current_sentence() {
            let (msg, style) = (vec![sentence.media_title.clone().into()], Style::default());
            let text = Text::from(Line::from(msg).patch_style(style));
            Paragraph::new(text)
                .block(Block::bordered().title("Media Title"))
                .centered()
                .render(area, buf);
        }
    }

    fn rend_sentence_info(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        exp_index: usize,
        sentences: Vec<Sentence>,
    ) {
        let vertical = Layout::vertical([Constraint::Percentage(10), Constraint::Percentage(10)]);
        let [top, top_middle] = vertical.areas(area);
        self.rend_media_title(top, buf);
    }

    fn rend_main(&mut self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(15),
            Constraint::Percentage(60),
            Constraint::Percentage(25),
        ]);
        let [expressions_area, sentences_area, info_area] = horizontal.areas(area);

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
                            SelectMode::Expressions => Style::default().light_yellow().bold(),
                            SelectMode::Sentences => Style::default(),
                        }),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED)
                        .fg(Color::White),
                );
            //.highlight_symbol("⇢ ");
            //.highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

            StatefulWidget::render(words, expressions_area, buf, &mut self.expressions_state);
        }

        // sentences area

        let mut sentence_items: Vec<ListItem> = Vec::new();
        if let Some(i) = self.selected_expression {
            let sentences = &self.expressions[i].sentences.clone();

            let sentences: Option<&Vec<Sentence>> = if let Some(sentences) = sentences {
                sentence_items = sentences
                    .iter()
                    .enumerate()
                    .map(|(i, sentence)| sentence.to_list_item(i))
                    .collect();

                Some(sentences)
            } else {
                None
            };

            let has_sentences = &sentence_items.is_empty();
            let sentences_list = List::new(sentence_items)
                .block(
                    Block::bordered()
                        .title(format!(
                            "{}'s Sentences",
                            &self.expressions[i].dict_word.clone()
                        ))
                        .style(match has_sentences {
                            true => Style::default().light_red().bold(),
                            false => match self.select_mode {
                                SelectMode::Expressions => Style::default().light_green().bold(),
                                SelectMode::Sentences => Style::default().light_yellow().bold(),
                            },
                        }),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED)
                        .fg(Color::White),
                );

            if let Some(sentences) = sentences {
                match self.select_mode {
                    SelectMode::Sentences => match self.expressions[i].selected_sentence {
                        Some(int) => {
                            if int > 0 {
                                self.rend_sentence_info(info_area, buf, i, sentences.to_vec());
                            } else {
                                self.render_blank_sentence_info_block(
                                    info_area,
                                    buf,
                                    has_sentences,
                                );
                            }
                        }
                        _ => {
                            self.render_blank_sentence_info_block(info_area, buf, has_sentences);
                        }
                    },
                    _ => {
                        self.render_blank_sentence_info_block(info_area, buf, has_sentences);
                    }
                }
            } else {
                self.render_blank_sentence_info_block(info_area, buf, has_sentences);
            }

            StatefulWidget::render(
                sentences_list,
                sentences_area,
                buf,
                &mut self.expressions[i].sentences_state,
            );
        }
    }

    fn render_blank_sentence_info_block(&self, area: Rect, buf: &mut Buffer, has_sentences: &bool) {
        Block::bordered()
            .title("Sentence Information")
            .style(match has_sentences {
                true => Style::default().light_red().bold(),
                false => match self.select_mode {
                    SelectMode::Expressions => Style::default().light_green().bold(),
                    SelectMode::Sentences => Style::default().light_yellow().bold(),
                },
            })
            .render(area, buf);
    }
}
