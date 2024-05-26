#![allow(clippy::enum_glob_use, clippy::wildcard_imports)]

mod app;
mod ui;
mod keybinds;

use std::{error::Error, io::stdout};
use color_eyre::config::HookBuilder;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    init_error_hooks()?;
    let term = init_terminal()?;
    app::AppState::new().run(term)?;

    restore_terminal()?;
    Ok(())
}

fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let term = Terminal::new(backend)?;
    Ok(term)
}

fn restore_terminal() -> color_eyre::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn init_error_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();

    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;

    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}
