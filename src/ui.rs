use crate::app::{AppState, Pages, SelectMode, Sentence};
use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, Paragraph},
};
use rayon::prelude::*;

impl Widget for &mut AppState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.selected_page {
            Pages::Main => {
                let layout = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ])
                .flex(layout::Flex::Center);
                let [help_area, main_area, info_area] = layout.areas(area);
                self.rend_top_main_area(help_area, buf);
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
                    .fg(Color::Green)
                    .add_modifier(Modifier::RAPID_BLINK),
            ),
            None => (
                format!("Words: [{}]", self.expressions.len()),
                Style::default(),
            ),
        };

        let text = Text::from(Line::from(msg).patch_style(style));
        let title = Line::from(vec![
            Span::styled("Information ", Color::Yellow),
            Span::styled("ⓘ ", Color::White),
        ]);

        Paragraph::new(text)
            .block(Block::bordered().title(title))
            .render(msg_area, buf);

        self.rend_err(err_area, buf);
    }

    fn rend_top_main_area(&mut self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ])
        .flex(layout::Flex::Center);
        let [left, _mid_left, _mid_right, right] = horizontal.areas(area);

        self.rend_main_keybinds(right, buf);
        self.rend_input_box(left, buf);
    }

    fn rend_err(&self, area: Rect, buf: &mut Buffer) {
        let (msg, style) = match &self.errors.len() {
            len if *len > 0 => (
                len.to_string(),
                Style::default().red().add_modifier(Modifier::RAPID_BLINK),
            ),
            0 => ("No Errors. :-)".to_string(), Style::default().green()),
            _ => unreachable!(),
        };

        let text = Text::from(Line::from(msg).patch_style(style));

        let title = Line::from(vec![
            Span::styled("Errors ", Color::Yellow),
            Span::styled("⚠  ", Color::White),
        ]);

        Paragraph::new(text)
            .block(Block::bordered().title(title))
            .render(area, buf);
    }

    fn rend_media_title(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(sentence) = &self.get_current_sentence() {
            let (msg, style) = (
                vec![sentence.media_title.clone().into()],
                Style::default().yellow(),
            );
            let text = Text::from(Line::from(msg).patch_style(style));
            Paragraph::new(text)
                .block(
                    Block::bordered().title(Line::styled("Media Title", Style::default().yellow())),
                )
                .style(Color::Green)
                .centered()
                .render(area, buf);
        }
    }

    fn rend_sentence_defs(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(i) = self.selected_expression {
            let definitions = match self.select_mode {
                SelectMode::Sentences => &self.expressions[i].definitions,
                _ => &self.expressions[i].definitions,
            };

            let def_items: Vec<ListItem> = definitions
                .par_iter()
                .enumerate()
                .map(|(i, def)| {
                    let mixed_line = Line::from(vec![
                        Span::styled(i.to_string(), Style::default().yellow()),
                        Span::styled(". ", Color::Green),
                        Span::styled(def, Style::default().white()),
                    ]);
                    ListItem::new(mixed_line)
                })
                .collect();

            let title = Line::from(vec![
                Span::styled(&self.expressions[i].dict_word, Style::default().yellow()),
                Span::styled("'s Definitions", Style::default().white()),
            ]);

            let defs = List::new(def_items).block(
                Block::bordered()
                    .title(title)
                    .style(Style::default().green()),
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
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Percentage(45),
            Constraint::Percentage(50),
        ])
        .flex(layout::Flex::Center);

        let [top_area, keybinds_area, about_area] = vertical.areas(area);
        self.rend_top_keybs_area(top_area, buf);
        self.rend_about(about_area, buf);

        let keybinds_horizontal = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ]);

        let [exp_kbs_area, sentences_kbs_area, notes_area, input_kbs_area] =
            keybinds_horizontal.areas(keybinds_area);

        self.rend_exp_keybinds(exp_kbs_area, buf);
        self.rend_sent_keybinds(sentences_kbs_area, buf);
        self.rend_notes_keybinds(notes_area, buf);
        self.rend_input_keybinds(input_kbs_area, buf);
    }

    fn rend_expressions(&mut self, area: Rect, buf: &mut Buffer) {
        let words: Vec<ListItem> = self
            .expressions
            .par_iter()
            .enumerate()
            .map(|(i, exp)| {
                let mut item = exp.to_list_item(i);
                if self
                    .notes_to_be_created
                    .sentences
                    .par_iter()
                    .find_map_first(|sent_obj| {
                        if sent_obj.parent_expression.dict_word == *exp.dict_word {
                            Some(())
                        } else {
                            None
                        }
                    })
                    .is_some()
                {
                    item = item.bg(Color::Green);
                }
                item
            })
            .collect();

        let words = List::new(words)
            .block(
                Block::bordered()
                    .title(match self.select_mode {
                        SelectMode::Expressions => {
                            Line::styled("Expressions", Style::default().white())
                        }
                        _ => Line::styled("Expressions", Style::default()).white(),
                    })
                    .style(match self.select_mode {
                        SelectMode::Expressions => Style::default().yellow(),
                        _ => Style::default().dim(),
                    }),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED).dim());
        //.highlight_symbol("⇢ ");
        //.highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        StatefulWidget::render(words, area, buf, &mut self.expressions_state);
    }

    fn rend_sentences(&mut self, sentences_area: Rect, info_area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::vertical([Constraint::Percentage(70), Constraint::Percentage(30)]);
        let [sentences_area, ntbc_area] = horizontal.areas(sentences_area);
        self.rend_notes_to_be_created(ntbc_area, buf);

        if self.expressions.is_empty() {
            return;
        }

        let mut sentence_items: Vec<ListItem> = Vec::new();
        if let Some(i) = self.selected_expression {
            if !&self.expressions[i].sentences.is_some() {
                return;
            }

            let selected_exp = &self.expressions[i].clone();
            let sentences = selected_exp.sentences.clone();
            let dict_word = selected_exp.dict_word.clone();
            let readings = selected_exp.readings.join("・").to_string();

            let sentences: Option<&Vec<Sentence>> = if let Some(sentences) = &sentences {
                sentence_items = sentences
                    .par_iter()
                    .enumerate()
                    .map(|(i, sentence)| {
                        let sent_obj = &sentence;
                        let item =
                            AppState::sentence_to_list_item(&sent_obj.sentence, &dict_word, i);
                        if self.notes_to_be_created.sentences.contains(sent_obj) {
                            return item.bg(Color::Green);
                        }
                        item
                    })
                    .collect();

                Some(sentences)
            } else {
                None
            };

            let sentence_title = Line::from(vec![
                Span::styled("「", Color::Green),
                Span::styled(readings, Style::default().yellow()),
                Span::styled("」", Color::Green),
                //Span::styled("∣", Color::Yellow),
                Span::styled(&dict_word, Style::default().white()),
            ]);

            let has_sentences = &sentence_items.is_empty();
            let sentences_list = List::new(sentence_items)
                .block(
                    Block::bordered()
                        .title(match self.select_mode {
                            SelectMode::Sentences => sentence_title,
                            _ => {
                                let current_sentence = self.get_current_sentence();
                                if current_sentence.is_some()
                                    && self
                                        .notes_to_be_created
                                        .sentences
                                        .contains(&current_sentence.unwrap())
                                {
                                    sentence_title
                                } else {
                                    Line::styled("Sentences", Style::default().light_red())
                                }
                            }
                        })
                        .style(match self.select_mode {
                            SelectMode::Expressions => {
                                let sentence = self.get_current_sentence();
                                if sentence.is_some()
                                    && self
                                        .notes_to_be_created
                                        .sentences
                                        .contains(&sentence.unwrap())
                                {
                                    Style::default().light_green().dim()
                                } else {
                                    Style::default().light_red().dim()
                                }
                            }
                            SelectMode::Sentences => Style::default().yellow(),
                            _ => Style::default().dim(),
                        }),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::REVERSED)
                        .fg(Color::White),
                );

            if sentences.is_some() {
                match self.select_mode {
                    SelectMode::Sentences => match self.expressions[i].selected_sentence {
                        Some(_int) => {
                            self.rend_sentence_info(info_area, buf);
                        }
                        _ => self.render_blank_sentence_info_block(info_area, buf, has_sentences),
                    },
                    // SelectMode::Ntbm => match self.notes_to_be_created.state.selected() {
                    //     Some(_int) => {
                    //         self.rend_sentence_info(info_area, buf);
                    //     }
                    //     _ => self.render_blank_sentence_info_block(info_area, buf, has_sentences),
                    // },
                    _ => self.render_blank_sentence_info_block(info_area, buf, has_sentences),
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

    fn rend_ntbc_kbs(&mut self, area: Rect, buf: &mut Buffer) {
        let mut extra = if self.select_mode == SelectMode::Ntbm {
            vec![
                "<Esc> ".red(),
                "Focus Expressions ".white(),
                "<C-Enter> ".light_green(),
                "Create Note(s) ".green(),
                "<D> ".light_red(),
                "Delete Sentence ".red(),
                "<I> ".light_yellow(),
                "Enter Note ID".yellow(),
            ]
        } else {
            vec!["<N> ".light_green(), "Focus Notes ".white()]
        };

        let (msg, style) = (vec![], Style::default());

        extra.extend(msg);

        let text = Text::from(Line::from(extra).patch_style(style));
        Paragraph::new(text)
            .block(
                Block::bordered()
                    .title(Line::styled(
                        "Note Keybinds",
                        Style::default().light_yellow(),
                    ))
                    .style(match self.select_mode {
                        SelectMode::Ntbm => Style::default().yellow(),
                        _ => Style::default(),
                    }),
            )
            .centered()
            .render(area, buf);
    }

    fn rend_notes_to_be_created(&mut self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::vertical([Constraint::Length(3), Constraint::Min(3)]);
        let [kbs_area, sentences_area] = horizontal.areas(area);

        self.rend_ntbc_kbs(kbs_area, buf);

        let i = self.notes_to_be_created.state.selected().unwrap_or(0);
        let selected_note: Option<Sentence> = if !&self.notes_to_be_created.sentences.is_empty() {
            Some(self.notes_to_be_created.sentences[i].clone())
        } else {
            None
        };

        let sentence_items: Vec<ListItem> = self
            .notes_to_be_created
            .sentences
            .par_iter()
            .enumerate()
            .map(|(i, sentence)| sentence.to_be_created_list_item(sentence, i))
            .collect();

        let span_title = if let Some(selected_note) = selected_note {
            Some(Line::from(vec![
                Span::styled(
                    selected_note.parent_expression.dict_word.clone(),
                    Style::default().light_green(),
                ),
                Span::styled("'s Sentence | ", Style::default().white()),
                Span::styled("Note ID: ", Style::default().white()),
                Span::styled(
                    selected_note
                        .parent_expression
                        .note_id
                        .map_or_else(|| "?".to_string(), |id| id.to_string()),
                    Style::default().light_green(),
                ),
            ]))
        } else {
            None
        };

        let sentences_list = List::new(sentence_items)
            .block(
                Block::bordered()
                    .title({
                        let current_sentence = self.get_current_sentence();
                        if self.select_mode == SelectMode::Ntbm && span_title.is_some() {
                            span_title.unwrap()
                        } else if current_sentence.is_some()
                            && self
                                .notes_to_be_created
                                .sentences
                                .contains(&current_sentence.unwrap())
                        {
                            Line::styled("Notes ", Style::default().light_green())
                        } else {
                            Line::styled("Notes ", Style::default().light_red())
                        }
                    })
                    .style(match self.select_mode {
                        SelectMode::Ntbm => Style::default().yellow(),
                        _ => {
                            let sentence = self.get_current_sentence();
                            if sentence.is_some()
                                && self
                                    .notes_to_be_created
                                    .sentences
                                    .contains(&sentence.unwrap())
                            {
                                Style::default().light_green().dim()
                            } else {
                                Style::default().light_red().dim()
                            }
                        }
                    }),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::White),
            );

        StatefulWidget::render(
            sentences_list,
            sentences_area,
            buf,
            &mut self.notes_to_be_created.state,
        );
    }

    fn rend_main(&mut self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(15),
            Constraint::Percentage(60),
            Constraint::Percentage(25),
        ])
        .flex(layout::Flex::Center);
        let [expressions_area, sentences_area, info_area] = horizontal.areas(area);
        self.rend_expressions(expressions_area, buf);
        self.rend_sentences(sentences_area, info_area, buf);
    }

    fn render_blank_sentence_info_block(&self, area: Rect, buf: &mut Buffer, has_sentences: &bool) {
        Block::bordered()
            .title("Sentence Information")
            .style(match has_sentences {
                true => Style::default().red().dim(),
                false => match self.select_mode {
                    SelectMode::Expressions => Style::default().green(),
                    SelectMode::Sentences => Style::default().yellow(),
                    _ => Style::default().dim(),
                },
            })
            .render(area, buf);
    }

    pub fn sentence_to_list_item<'a>(sentence: &'a str, word: &'a str, i: usize) -> ListItem<'a> {
        let (start, end) = sentence
            .match_indices(word)
            .next()
            .map(|(start, word)| (start, start + word.len()))
            .unwrap_or((0, 0));

        let before_word = &sentence[..start];
        let found_word = &sentence[start..end];
        let after_word = &sentence[end..];

        let mixed_line = Line::from(vec![
            //Span::styled("|", Color::Green),
            Span::styled(i.to_string(), Style::default().yellow()),
            Span::styled(". ", Color::Green),
            Span::styled(before_word, Color::White),
            Span::styled(found_word, Style::default().yellow()),
            Span::styled(after_word, Color::White),
        ]);

        ListItem::new(mixed_line)
    }
}
