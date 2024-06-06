use crate::app::{AppState, Pages, SelectMode, Sentence};
use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, /* Padding */ Paragraph},
};

impl Widget for &mut AppState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.selected_page {
            Pages::Main => {
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
            Pages::Help => {
                self.rend_help_page(area, buf);
            }
            Pages::Splice => {
                //self.rend_splice_page(area, buf);
            }
        }
    }
}

impl AppState {
    fn rend_info_area(&self, area: Rect, buf: &mut Buffer) {
        let horizontal =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);

        let [msg_area, err_area] = horizontal.areas(area);

        let (msg, style) = match &self.info.msg {
            Some(msg) => (
                msg.clone(),
                Style::default()
                    .bold()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::RAPID_BLINK),
            ),
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

        self.rend_main_keybinds(right, buf);
        self.rend_input_box(left, buf);
    }

    fn rend_err(&self, area: Rect, buf: &mut Buffer) {
        let (msg, style) = match &self.err_msg {
            Some(msg) => (
                msg.clone(),
                Style::default()
                    .light_red()
                    .bold()
                    .add_modifier(Modifier::RAPID_BLINK),
            ),
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
                .style(Color::Yellow)
                .centered()
                .render(area, buf);
        }
    }

    fn rend_sentence_defs(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(i) = self.selected_expression {
            let definitions = &self.expressions[i].definitions;

            let def_items = definitions.iter().enumerate().map(|(i, def)| {
                let (msg, style) = (
                    Span::from(format!("{}. {}", i, def)),
                    Style::default().white(),
                );
                let line = Line::from(msg).patch_style(style);
                ListItem::new(line)
            });

            let defs = List::new(def_items).block(
                Block::bordered()
                    .title(format!("{}'s Definitions", &self.expressions[i].dict_word))
                    .style(Style::default().light_blue()),
            );

            ratatui::widgets::Widget::render(&defs, area, buf);
        }
    }

    fn rend_sentence_info(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(i) = self.selected_expression {
            let vertical = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length((self.expressions[i].definitions.len() + 2) as u16),
            ]);
            let [top, top_middle] = vertical.areas(area);
            self.rend_media_title(top, buf);
            self.rend_sentence_defs(top_middle, buf)
        }
    }

    fn rend_help_page(&mut self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);

        let [keybinds_area, about_area] = vertical.areas(area);
        let keybinds_horizontal = Layout::horizontal([
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
        ]);
        let [exp_kbs_area, sentences_kbs_area, input_kbs_area] =
            keybinds_horizontal.areas(keybinds_area);
        self.rend_keybinds(exp_kbs_area, buf);

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
                            SelectMode::Expressions => Style::default().yellow().bold(),
                            _ => Style::default(),
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
                            "({}) {}'s Sentences",
                            &self.expressions[i].readings.join("・"),
                            &self.expressions[i].dict_word.clone()
                        ))
                        .style(match has_sentences {
                            true => Style::default().red().bold(),
                            false => match self.select_mode {
                                SelectMode::Expressions => Style::default().light_green().bold(),
                                SelectMode::Sentences => Style::default().light_yellow().bold(),
                                _ => Style::default(),
                            },
                        }),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED)
                        .fg(Color::White),
                );

            if sentences.is_some() {
                match self.select_mode {
                    SelectMode::Sentences => match self.expressions[i].selected_sentence {
                        Some(_int) => {
                            self.rend_sentence_info(info_area, buf);
                            //self.render_blank_sentence_info_block(info_area, buf, has_sentences);
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
                true => Style::default().red().bold(),
                false => match self.select_mode {
                    SelectMode::Expressions => Style::default().light_green().bold(),
                    SelectMode::Sentences => Style::default().light_yellow().bold(),
                    _ => Style::default(),
                },
            })
            .render(area, buf);
    }
}
