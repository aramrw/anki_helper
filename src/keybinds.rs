use crossterm::event::{/*self, Event */ KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
//use ratatui::prelude::*;
use crate::app::{AppState, Pages, SelectMode, Sentence};
//use crate::audio::{decode_audio_bytes, trim_samples_from_start};
use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, ListState, Paragraph},
};
use std::io;

#[derive(Default, Debug, PartialEq)]
pub enum KeybindSections {
    #[default]
    Expressions,
    Sentences,
    Notes,
    Input,
}

#[derive(Default)]
pub struct Keybinds {
    pub exp_titles: Vec<String>,
    pub sent_titles: Vec<String>,
    pub input_titles: Vec<String>,
    pub note_titles: Vec<String>,
    pub exp_state: ListState,
    pub sent_state: ListState,
    pub input_state: ListState,
    pub note_state: ListState,
    pub exp_abouts: Vec<String>,
    pub sent_abouts: Vec<String>,
    pub input_abouts: Vec<String>,
    pub note_abouts: Vec<String>,
    pub selected_section: KeybindSections,
}

impl AppState {
    pub async fn handle_keybinds(&mut self, key: KeyEvent) -> io::Result<()> {
        if self.expressions.is_empty() {
            self.update_error_msg("words.txt Error", "The file is empty!".to_string());
            self.read_words_file().unwrap();
            return Ok(());
        }
        match self.selected_page {
            Pages::Main => match self.select_mode {
                SelectMode::Expressions if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('I') => self.select_mode = SelectMode::Input,
                    KeyCode::Char('Y') => self.handle_copy_to_input(),
                    KeyCode::Char('D') => {
                        if let Some(i) = self.selected_expression {
                            let current_wrd = &self.expressions[i].dict_word.clone();
                            match self.delete_words_from_file(&vec![current_wrd.to_string()]) {
                                Ok(_) => {
                                    self.info.msg =
                                        format!("Deleted: {} from words.txt", &current_wrd).into()
                                }
                                Err(err) => self.update_error_msg(
                                    "Err Deleting {} from words.txt: {}",
                                    err.to_string(),
                                ),
                            }
                        }
                    }
                    KeyCode::Enter if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(i) = self.selected_expression {
                            self.expressions[i].sentences = None;
                            self.expressions[i].exact_search = false;
                            self.select_mode = SelectMode::Sentences;
                            if self.expressions[i].sentences.is_none() {
                                self.expressions[i].sentences_state.select(Some(0));
                                self.fetch_sentences().await;
                            }
                        }
                    }
                    KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(i) = self.selected_expression {
                            self.expressions[i].sentences = None;
                            self.expressions[i].exact_search = true;
                            self.select_mode = SelectMode::Sentences;
                            if self.expressions[i].sentences.is_none() {
                                self.expressions[i].sentences_state.select(Some(0));
                                self.fetch_sentences().await;
                            }
                        }
                    }
                    KeyCode::Down => self.select_next_exp(),
                    KeyCode::Up => self.select_prev_exp(),
                    _ => {}
                },
                SelectMode::Sentences if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('L') => self.open_website_link(),
                    KeyCode::Char('P') => {
                        if let Err(err) = self.play_audio().await {
                            self.update_error_msg("Error Playing Audio", err.to_string());
                        }
                    }
                    KeyCode::Enter if !key.modifiers.contains(KeyModifiers::CONTROL) => self.check_notes_or_push(),
                    KeyCode::Esc => self.reset_sentences_index(),
                    KeyCode::Up => {
                        if self.selected_page == Pages::Splice {
                            return Ok(());
                        }
                        self.select_prev_sentence()
                    }
                    KeyCode::Down => {
                        if self.selected_page == Pages::Splice {
                            return Ok(());
                        }
                        self.select_next_sentence()
                    }
                    _ => {}
                },
                SelectMode::Input if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('P') => self.handle_paste(),
                    KeyCode::Esc => self.select_mode = SelectMode::Expressions,
                    KeyCode::Enter => self.confirm_search_query().await,
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    KeyCode::Char(input_char) => self.enter_char(input_char),
                    _ => {}
                },
                SelectMode::Ntbm if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if self.notes_to_be_created.sentences.is_empty() {
                            self.err_msg = Some("Error: You must have at least 1 sentence selected to update or create a Note".to_string());
                            return Ok(());
                        }
                        self.update_anki_cards().await;
                    }
                    KeyCode::Esc => self.select_mode = SelectMode::Expressions,
                    KeyCode::Up => self.select_prev_note(),
                    KeyCode::Down => self.select_next_note(),
                    KeyCode::Char('D') => self.delete_note(),
                    _ => {}
                },
                // main page global keybinds
                _ => match key.code {
                    KeyCode::Char('H') => {
                        if self.keybinds.exp_state.selected().is_none() {
                            self.keybinds.exp_state.select(Some(0));
                        };
                        self.selected_page = Pages::Help
                    }
                    KeyCode::Char('N') => {
                        if self.notes_to_be_created.state.selected().is_none() {
                            self.notes_to_be_created.state.select(Some(0));
                        }
                        self.select_mode = SelectMode::Ntbm
                    }
                    KeyCode::Char('M') => self.selected_page = Pages::Main,
                    KeyCode::Char('R') => self.restart_program(),
                    _ => {}
                },
            },

            Pages::Help => {
                match self.keybinds.selected_section {
                    KeybindSections::Expressions if key.kind == KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Down => self.select_next_keybind(KeybindSections::Expressions),
                            KeyCode::Up => self.select_prev_keybind(KeybindSections::Expressions),
                            _ => {}
                        }
                    }
                    KeybindSections::Sentences if key.kind == KeyEventKind::Press => match key.code
                    {
                        KeyCode::Down => self.select_next_keybind(KeybindSections::Sentences),
                        KeyCode::Up => self.select_prev_keybind(KeybindSections::Sentences),
                        _ => {}
                    },
                    KeybindSections::Notes if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Down => self.select_next_keybind(KeybindSections::Notes),
                        KeyCode::Up => self.select_prev_keybind(KeybindSections::Notes),
                        _ => {}
                    },
                    KeybindSections::Input if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Down => self.select_next_keybind(KeybindSections::Input),
                        KeyCode::Up => self.select_prev_keybind(KeybindSections::Input),
                        _ => {}
                    },
                    _ => {}
                }

                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('E') => {
                            self.keybinds.selected_section = KeybindSections::Expressions;
                        }
                        KeyCode::Char('S') => {
                            if self.keybinds.sent_state.selected().is_none() {
                                self.keybinds.sent_state.select(Some(0));
                            };
                            self.keybinds.selected_section = KeybindSections::Sentences;
                        }
                        KeyCode::Char('N') => {
                            if self.keybinds.note_state.selected().is_none() {
                                self.keybinds.note_state.select(Some(0));
                            };
                            self.keybinds.selected_section = KeybindSections::Notes;
                        }
                        KeyCode::Char('I') => {
                            if self.keybinds.input_state.selected().is_none() {
                                self.keybinds.input_state.select(Some(0));
                            };
                            self.keybinds.selected_section = KeybindSections::Input;
                        }
                        KeyCode::Char('G') => self.open_github(),
                        KeyCode::Esc => self.selected_page = Pages::Main,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

pub fn check_notes_or_push(&mut self) {
    if let Some(sentence) = self.get_current_sentence() {
        let mut found = false;

        let mut new_sentences: Vec<Sentence> = self
            .notes_to_be_created
            .sentences
            .iter()
            .map(|ntb_sent| {
                if ntb_sent.parent_expression.dict_word == sentence.parent_expression.dict_word {
                    found = true;
                    sentence.clone()
                } else {
                    ntb_sent.clone()
                }
            })
            .collect();

        if !found {
            new_sentences.push(sentence.clone());
        }

        self.notes_to_be_created.sentences = new_sentences;
    }

    self.select_mode = SelectMode::Expressions;
}


    pub fn delete_note(&mut self) {
        if self.notes_to_be_created.sentences.is_empty() {
            return;
        }
        let i = self.notes_to_be_created.state.selected().unwrap_or(0);
        self.notes_to_be_created.sentences.remove(i);
        if self.notes_to_be_created.sentences.is_empty() {
            self.select_mode = SelectMode::Expressions;
            return;
        }
        if i == 0 {
            return;
        }

        self.notes_to_be_created.state.select(Some(i - 1));
    }

    pub fn reset_sentences_index(&mut self) {
        if let Some(i) = self.selected_expression {
            self.expressions[i].selected_sentence = None;
            self.expressions[i].sentences_state.select(None);
        }
        self.select_mode = SelectMode::Expressions
    }

    pub fn select_prev_note(&mut self) {
        let len = self.notes_to_be_created.sentences.len();
        if len == 0 {
            return;
        }
        let note_index = match self.notes_to_be_created.state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.notes_to_be_created.state.select(Some(note_index));
    }

    pub fn select_next_note(&mut self) {
        let len = self.notes_to_be_created.sentences.len();
        if len == 0 {
            return;
        }
        let note_index = match self.notes_to_be_created.state.selected() {
            Some(i) => {
                if i == len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.notes_to_be_created.state.select(Some(note_index));
    }

    pub fn select_prev_sentence(&mut self) {
        if let Some(exp_index) = self.selected_expression {
            let selected_exp = &self.expressions[exp_index];
            if let Some(sentences) = &selected_exp.sentences {
                let sentence_index = match selected_exp.sentences_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            sentences.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => selected_exp.selected_sentence.unwrap_or(0),
                };
                self.expressions[exp_index].selected_sentence = Some(sentence_index);
                self.expressions[exp_index]
                    .sentences_state
                    .select(Some(sentence_index));
            }
        }
    }

    pub fn select_next_sentence(&mut self) {
        if let Some(exp_index) = self.selected_expression {
            let selected_exp = &self.expressions[exp_index];
            if let Some(sentences) = &selected_exp.sentences {
                if sentences.is_empty() {
                    return;
                }
                let sentence_index = match selected_exp.sentences_state.selected() {
                    Some(i) => {
                        if i == sentences.len().saturating_sub(1) {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => selected_exp.selected_sentence.unwrap_or(0),
                };
                self.expressions[exp_index].selected_sentence = Some(sentence_index);
                self.expressions[exp_index]
                    .sentences_state
                    .select(Some(sentence_index));
            }
        }
    }

    pub fn select_prev_exp(&mut self) {
        let i = match self.expressions_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.expressions.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => self.selected_expression.unwrap_or(0),
        };

        self.selected_expression = Some(i);
        self.expressions_state.select(Some(i));
    }

    pub fn select_next_exp(&mut self) {
        let i = match self.expressions_state.selected() {
            Some(i) => {
                if i == self.expressions.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.selected_expression.unwrap_or(0),
        };

        self.selected_expression = Some(i);
        self.expressions_state.select(Some(i));
    }

    pub fn rend_main_keybinds(&self, area: Rect, buf: &mut Buffer) {
        let (msg, style) = (
            vec![
                "<H> ".green(),
                "Help Page ".into(),
                "<R> ".red(),
                "Restart Program ".into(),
                // "<S> ".white(),
                // "Audio Cutter".into(),
            ],
            Style::default(),
        );

        let text = Text::from(Line::from(msg).patch_style(style));
        Paragraph::new(text)
            .block(Block::bordered().title(Line::styled("Keybinds", Style::default().yellow())))
            .centered()
            .render(area, buf);
    }

    pub fn rend_top_keybs_area(&mut self, area: Rect, buf: &mut Buffer) {
        let (msg, style) = (
            vec![
                "<Esc> ".red(),
                "Go Back ".into(),
                "<G> ".green(),
                "Github Repo ".into(),
            ],
            Style::default(),
        );

        let text = Text::from(Line::from(msg).patch_style(style));
        Paragraph::new(text)
            .block(Block::bordered().title(Line::styled("Keybinds", Style::default().yellow())))
            .centered()
            .render(area, buf);
    }

    pub fn rend_exp_keybinds(&mut self, area: Rect, buf: &mut Buffer) {
        let kb_titles: Vec<ListItem> = self
            .keybinds
            .exp_titles
            .iter()
            .enumerate()
            .map(|(i, kb)| Keybinds::to_list_item(kb, i))
            .collect();

        let kbs = List::new(kb_titles)
            .block(Block::bordered().title("Expression Keybinds <E>").style(
                match self.keybinds.selected_section {
                    KeybindSections::Expressions => Style::default().yellow(),
                    _ => Style::default().dim(),
                },
            ))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::White),
            );

        StatefulWidget::render(kbs, area, buf, &mut self.keybinds.exp_state);
    }

    pub fn rend_sent_keybinds(&mut self, area: Rect, buf: &mut Buffer) {
        let kb_titles: Vec<ListItem> = self
            .keybinds
            .sent_titles
            .iter()
            .enumerate()
            .map(|(i, kb)| Keybinds::to_list_item(kb, i))
            .collect();

        let kbs = List::new(kb_titles)
            .block(Block::bordered().title("Sentence Keybinds <S>").style(
                match self.keybinds.selected_section {
                    KeybindSections::Sentences => Style::default().yellow(),
                    _ => Style::default().dim(),
                },
            ))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::White),
            );

        StatefulWidget::render(kbs, area, buf, &mut self.keybinds.sent_state);
    }

    pub fn rend_notes_keybinds(&mut self, area: Rect, buf: &mut Buffer) {
        let kb_titles: Vec<ListItem> = self
            .keybinds
            .note_titles
            .iter()
            .enumerate()
            .map(|(i, kb)| Keybinds::to_list_item(kb, i))
            .collect();

        let kbs = List::new(kb_titles)
            .block(Block::bordered().title("Notes Keybinds <N>").style(
                match self.keybinds.selected_section {
                    KeybindSections::Notes => Style::default().yellow(),
                    _ => Style::default().dim(),
                },
            ))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::White),
            );

        StatefulWidget::render(kbs, area, buf, &mut self.keybinds.note_state);
    }

    pub fn rend_input_keybinds(&mut self, area: Rect, buf: &mut Buffer) {
        let kb_titles: Vec<ListItem> = self
            .keybinds
            .input_titles
            .iter()
            .enumerate()
            .map(|(i, kb)| Keybinds::to_list_item(kb, i))
            .collect();

        let kbs = List::new(kb_titles)
            .block(Block::bordered().title("Search Keybinds <I>").style(
                match self.keybinds.selected_section {
                    KeybindSections::Input => Style::default().yellow(),
                    _ => Style::default().dim(),
                },
            ))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::White),
            );

        StatefulWidget::render(kbs, area, buf, &mut self.keybinds.input_state);
    }

    pub fn rend_about(&mut self, area: Rect, buf: &mut Buffer) {
        let i = match self.keybinds.selected_section {
            KeybindSections::Expressions => self.keybinds.exp_state.selected().unwrap_or(0),
            KeybindSections::Sentences => self.keybinds.sent_state.selected().unwrap_or(0),
            KeybindSections::Notes => self.keybinds.note_state.selected().unwrap_or(0),
            KeybindSections::Input => self.keybinds.input_state.selected().unwrap_or(0),
        };

        let (about, _style) = match &self.keybinds.selected_section {
            KeybindSections::Expressions => (&self.keybinds.exp_abouts[i], Style::default()),
            KeybindSections::Sentences => (&self.keybinds.sent_abouts[i], Style::default().white()),
            KeybindSections::Notes => (&self.keybinds.note_abouts[i], Style::default().white()),
            KeybindSections::Input => (&self.keybinds.input_abouts[i], Style::default().white()),
        };

        let lines: Vec<Line> = about
            .lines()
            .enumerate()
            .map(|(i, line)| {
                if i == 0 {
                    return Line::styled(line, Style::default().yellow().underlined());
                }
                Line::styled(line, Style::default())
            })
            .collect();

        Paragraph::new(lines)
            .block(
                Block::bordered()
                    .title(Line::styled(
                        format!("{:?} Help & Explanations", self.keybinds.selected_section),
                        Style::default().yellow(),
                    ))
                    .style(Style::default()),
            )
            .centered()
            .render(area, buf);
    }

    fn get_kb_section_state_and_len(
        &mut self,
        section: KeybindSections,
    ) -> (usize, &mut ListState) {
        let mut _len = 0;
        let state = match section {
            KeybindSections::Expressions => {
                _len = self.keybinds.exp_titles.len();
                &mut self.keybinds.exp_state
            }
            KeybindSections::Sentences => {
                _len = self.keybinds.sent_titles.len();
                &mut self.keybinds.sent_state
            }
            KeybindSections::Notes => {
                _len = self.keybinds.note_titles.len();
                &mut self.keybinds.note_state
            }
            KeybindSections::Input => {
                _len = self.keybinds.input_titles.len();
                &mut self.keybinds.input_state
            }
        };

        (_len, state)
    }

    pub fn select_prev_keybind(&mut self, section: KeybindSections) {
        let (_len, state) = self.get_kb_section_state_and_len(section);

        let state_i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    _len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        state.select(Some(state_i));
    }

    pub fn select_next_keybind(&mut self, section: KeybindSections) {
        let (_len, state) = self.get_kb_section_state_and_len(section);

        let state_i = match state.selected() {
            Some(i) => {
                if i == _len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.select(Some(state_i));
    }
}

impl Keybinds {
    pub fn to_list_item(text: &str, _i: usize) -> ListItem {
        let (msg, style) = (
            vec!["<".white(), text.yellow(), ">".white()],
            Style::default(),
        );
        let line = Line::from(msg).patch_style(style);
        ListItem::new(line.centered())
    }

    pub fn new() -> Self {
        // exp

        let exp_titles = ["Enter", "C-Enter", "I", "Y", "D", "Up", "Down"]
            .iter()
            .map(|kb| kb.to_string())
            .collect();

        let exp_abouts = [
                "Fetches Sentences\n‎\nSentences may include, or exactly match the selected Expression in one of its forms.\nDepending on the word's rarity, either it's kanji form, or it's kana reading may provide more accurate results.\nSee `<C-Enter>` for more information on sentence accuracy.\n‎\nIf no sentences are found from Immersion Kit, it will fetch sentences from Massif.la.\nMassif.la sentences don't contain audio or images.\n(WIP) You can set `\"tts\": true` in your config.json to generate audio for the sentence.",
                "[Ctrl + Enter] - Enables `Exact Search` for Immersion Kit Search Results\n‎\nThis will find sentences that contain a 1 to 1 match of the selected Expression.\n‎\nThis means that it will not try to match the Expressions kana reading.\nOr if the Expression is a verb, it will not recognize it's conjugated forms.\n‎\nIf no sentences are found from Immersion Kit with `Exact Search` enabled, it will still fetch from Massif.la (with `Exact Search` disabled).",
                "Focuses the Search Box\n‎\nPress <I> to see Search Box keybinds.",
                "Copies Selected Expression into Input Box\n‎\nPress <I> to see Search Box keybinds.",
                "Deletes the Selected Expression\n‎\nThis will also remove the expression from your words.txt file.\nYou can set `\"del_word\": true` in your config.json to automatically delete selected Expressions from your words.txt after updating their Anki Notes.",
                "Selects the Previous Expression\n‎\nFocuses the Previous Expression in the Expressions List.",
                "Selects the Next Expression\n‎\nFocuses the Next Expression in the Expressions List.",
            ]
            .iter()
            .map(|ab| ab.to_string())
            .collect();

        // sent

        let sent_titles = ["P", "L", "Esc", "Up", "Down"]
            .iter()
            .map(|kb| kb.to_string())
            .collect();

        let sent_abouts =[
                "Plays the Sentence's Audio\n‎\nMassif.la sentences don't contain audio, so nothing will play.\n(WIP) You can set `\"tts\": true` in your config.json to generate audio for the sentence.",
                "Opens Sentence in the Default Browser\n‎\nThis will take you to either Immersion Kit, or Massif.la's website with the sentence pasted into the searchbar.",
                "Focuses Expressions List\n‎\nUnfocuses the Sentences List & Focuses the Expressions List.",
                "Selects the Previous Sentence\n‎\nFocuses the Previous Expression in the Sentences List.",
                "Selects the Next Sentence\n‎\nFocuses the Next Sentence in the Sentences List.",
            ]
            .iter()
            .map(|ab| ab.to_string())
            .collect();

        // notes

        let note_titles = ["C-Enter", "D", "N", "Esc"]
            .iter()
            .map(|kb| kb.to_string())
            .collect();

        let note_abouts = ["[Ctrl + Enter] - Update Notes\n‎\nFinds, checks, then updates any Anki Notes that contain the selected Expressions.\nIf the selected Sentence was fetched from Massif.la, it will only update the Sentence field specified in your config.json.\nOtherwise it will update the Sentence, and Audio fields.\nSome entries on Immersion Kit do not contain an image file (ie. Skyrim).\nIf an image file exists, it will be added as well.\n‎\nWarning: Overwrites existing data in the Anki fields specified in your config.json except the Audio & Image fields, those will get appended to.",
            "Deletes the Selected Sentence\n‎\nRemoves the sentence from the Notes list.",
            "Focuses the Notes Section\n‎\nFocuses the Notes section if it is not already focused.",
            "Focuses to the Expressions Section\n‎\nFocuses the Expressions section if focused on the Notes section."
            ]
            .iter()
            .map(|ab| ab.to_string())
            .collect();

        // input

        let input_titles = ["Enter", "P", "Left", "Right", "Backspace"]
            .iter()
            .map(|kb| kb.to_string())
            .collect();

        // \nYou can update a specific Anki Note by entering the Anki Note ID into the Search Box.\nThis can be useful in rare cases where Anki may not be able to find the Note containing the selected Expression.

        let input_abouts = ["Submits the Current Input\n‎\nYou can jump to a specific Expression by entering it's List number.\nYou can also jump to a specific Expression by entering the Expression. Note that it must be an exact match.", 
            "Pastes from Clipboard\n‎\nPastes the current copied text from the Clipboard into the Search Box.",
            "Selects the Previous Character\n‎\nSelects the Previous Character of the Text in the Search Box.",
            "Selects the Next Character\n‎\nFocuses the Next Char of the Text in the Search Box.",
            "Deletes the Previous Character\n‎\nDeletes the Previous Character of the Text in the Search Box."
        ]
            .iter()
            .map(|ab| ab.to_string())
            .collect();

        Self {
            exp_titles,
            sent_titles,
            note_titles,
            input_titles,
            exp_state: ListState::default(),
            sent_state: ListState::default(),
            note_state: ListState::default(),
            input_state: ListState::default(),
            exp_abouts,
            sent_abouts,
            note_abouts,
            input_abouts,
            selected_section: KeybindSections::Expressions,
        }
    }
}
