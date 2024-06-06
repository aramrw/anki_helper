// use crate::app::AppState;
// use ratatui::{
//     prelude::*,
//     widgets::{Block, Paragraph, RenderDirection, Sparkline},
// };
//
// use std::io::{Cursor, Read, Seek};
// use symphonia::core::audio::{AudioBuffer, AudioBufferRef, Signal};
// use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
// use symphonia::core::errors::Error;
// use symphonia::core::formats::{FormatOptions, Track};
// use symphonia::core::io::MediaSourceStream;
// use symphonia::core::meta::MetadataOptions;
// use symphonia::default::get_probe;
//
//
// pub fn decode_audio_bytes(input_bytes: Vec<u8>) -> Result<(Vec<i16>, u32, u8), Error> {
//     let cursor = Cursor::new(input_bytes);
//     let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
//
//     // Use the default probe to detect the format
//     let mut probed = get_probe().format(
//         &Default::default(),
//         mss,
//         &FormatOptions::default(),
//         &MetadataOptions::default(),
//     ).expect("Failed to probe format");
//
//     // Check if probed object has tracks
//     if probed.format.tracks().is_empty() {
//         panic!("No tracks found in the audio data.");
//     }
//
//     // Get the first audio track
//     let track = probed
//         .format
//         .tracks()
//         .iter()
//         .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
//         .ok_or_else(|| panic!("No audio track found"));
//
//     // Create a decoder for the track
//     let mut decoder =
//         symphonia::default::get_codecs().make(&track.unwrap().codec_params, &DecoderOptions::default())
//         .expect("Failed to create decoder");
//
//     let mut samples = Vec::new();
//     let mut sample_rate = 0;
//     let mut channels = 0;
//
//     // Check if packets are being read
//     let mut packet_count = 0;
//     while let Ok(packet) = probed.format.next_packet() {
//         packet_count += 1;
//         match decoder.decode(&packet) {
//             Ok(decoded) => {
//                 match decoded {
//                     AudioBufferRef::S16(buffer) => {
//                         // Set sample rate and channels if not already set
//                         if sample_rate == 0 {
//                             sample_rate = buffer.spec().rate;
//                             channels = buffer.spec().channels.count() as u8;
//                             println!("Sample rate: {}, Channels: {}", sample_rate, channels);
//                         }
//
//                         // Append samples from each channel
//                         for frame in buffer.chan(0).iter() {
//                             samples.push(*frame);
//                         }
//                     }
//                     AudioBufferRef::F32(buffer) => {
//                         // Convert F32 to i16
//                         if sample_rate == 0 {
//                             sample_rate = buffer.spec().rate;
//                             channels = buffer.spec().channels.count() as u8;
//                             println!("Sample rate: {}, Channels: {}", sample_rate, channels);
//                         }
//
//                         for frame in buffer.chan(0).iter() {
//                             samples.push((*frame * i16::MAX as f32) as i16);
//                         }
//                     }
//                     AudioBufferRef::S32(buffer) => {
//                         // Convert S32 to i16
//                         if sample_rate == 0 {
//                             sample_rate = buffer.spec().rate;
//                             channels = buffer.spec().channels.count() as u8;
//                             println!("Sample rate: {}, Channels: {}", sample_rate, channels);
//                         }
//
//                         for frame in buffer.chan(0).iter() {
//                             samples.push((*frame >> 16) as i16); // Downsample 32-bit to 16-bit
//                         }
//                     }
//                     _ => {
//                         panic!("Decoded audio buffer is not a supported format");
//                     }
//                 }
//             },
//             Err(e) => {
//                 panic!("Error decoding packet: {:?}", e);
//             }
//         }
//     }
//
//     // Check if any packets were read
//     if packet_count == 0 {
//         panic!("No packets were read from the media source");
//     }
//
//     // Check if any samples were actually collected
//     if samples.is_empty() {
//         panic!("No audio samples were collected.");
//     }
//
//     Ok((samples, sample_rate, channels))
// }
//
// pub fn trim_samples_from_start(samples: Vec<i16>, sample_rate: u32, channels: u8) -> Vec<i16> {
//     let start_ms = 500; // 500 milliseconds
//     let start_sample = (start_ms as usize * sample_rate as usize * channels as usize) / 1000;
//     samples[start_sample..].to_vec()
// }
//
// pub fn trim_samples_from_end(samples: Vec<i16>, sample_rate: u32, channels: u8) -> Vec<i16> {
//     let end_ms = 500; // 500 milliseconds
//     let end_sample =
//         samples.len() - (end_ms as usize * sample_rate as usize * channels as usize) / 1000;
//     samples[..end_sample].to_vec()
// }
//
// impl AppState {
//     pub fn rend_splice_page(&self, area: Rect, buf: &mut Buffer) {
//         let vertical = Layout::vertical([
//             Constraint::Length(3),
//             Constraint::Min(3),
//             Constraint::Length(3),
//         ]);
//         let [top_bar, middle_area, bottom_bar] = vertical.areas(area);
//         self.render_top_bar(top_bar, buf);
//         self.render_main_editor(middle_area, buf);
//     }
//
//     fn render_main_editor(&self, area: Rect, buf: &mut Buffer) {
//         let vertical = Layout::vertical([Constraint::Percentage(80)]);
//
//         let [display] = vertical.areas(area);
//
//         //Block::new().style(Style::default()).render(left, buf);
//         //Block::new().style(Style::default()).render(right, buf);
//
//         self.render_audio_wave(display, buf);
//     }
//
//     fn render_audio_wave(&self, area: Rect, buf: &mut Buffer) {
//         if let Some(sentence) = self.get_current_sentence() {
//             if let Some(bytes) = sentence.audio_data {
//                 let data: Vec<u64> = bytes.iter().map(|&b| b as u64).collect();
//                 Sparkline::default()
//                     .block(Block::bordered().title("Waveform"))
//                     .data(&data)
//                     .direction(RenderDirection::LeftToRight)
//                     .style(Style::default().red().on_white())
//                     .render(area, buf);
//             }
//         }
//     }
//
//     fn render_top_bar(&self, area: Rect, buf: &mut Buffer) {
//         if let Some(sentence) = self.get_current_sentence() {
//             let media_title = sentence.media_title;
//             let horizontal = Layout::horizontal([
//                 Constraint::Length((media_title.len() + 2) as u16),
//                 Constraint::Min(5),
//                 Constraint::Length(17),
//                 Constraint::Min(15),
//             ]);
//             let [media_title_area, sentence_area, play_button, keybinds_area] =
//                 horizontal.areas(area);
//
//             Paragraph::new(Span::from(media_title).style(Color::White))
//                 .block(Block::bordered().title("Media Title"))
//                 .fg(Color::Yellow)
//                 .render(media_title_area, buf);
//
//             Paragraph::new(Span::from(sentence.sentence).style(Color::White))
//                 .block(Block::bordered().title("Sentence"))
//                 .fg(Color::Yellow)
//                 .render(sentence_area, buf);
//
//             // play btn
//             let (mut msg, mut style) = (vec!["<P> ".blue(), "Play Audio".into()], Style::default());
//             let mut text = Text::from(Line::from(msg)).patch_style(style);
//             Paragraph::new(text)
//                 .block(Block::bordered())
//                 .fg(Color::LightBlue)
//                 .render(play_button, buf);
//
//             (msg, style) = (
//                 vec![
//                     "<Left> ".blue(),
//                     "Trim Less ".white(),
//                     "<Right> ".blue(),
//                     "Trim More ".white(),
//                     "<R> ".light_red(),
//                     "Reset ".white(),
//                     "<C-Enter> ".light_green(),
//                     "Confirm".white(),
//                 ],
//                 Style::default(),
//             );
//             text = Text::from(Line::from(msg)).patch_style(style);
//             Paragraph::new(text)
//                 .block(Block::bordered())
//                 .centered()
//                 .fg(Color::LightCyan)
//                 .render(keybinds_area, buf);
//         }
//     }
// }
