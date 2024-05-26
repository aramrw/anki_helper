#![allow(clippy::enum_glob_use, clippy::wildcard_imports)]

use std::{error::Error, io, io::stdout};

use color_eyre::config::HookBuilder;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    init_error_hooks()?;
    let term = init_terminal()?;

    restore_terminal()?;
    Ok(())
}

