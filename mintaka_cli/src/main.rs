use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{Event, KeyEvent};
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::crossterm::{event, ExecutableCommand};
use ratatui::{Frame, Terminal};
use std::io;
use std::io::stdout;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(ui)?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            match key {
                KeyEvent { .. } => {}
            }
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame) {
}
