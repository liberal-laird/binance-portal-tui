use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug)]
pub enum EventType {
    Input(KeyCode),
    Tick,
    #[allow(dead_code)]
    Refresh,
}

pub struct EventHandler {
    #[allow(dead_code)]
    pub input_mode: InputMode,
    pub tick_rate: Duration,
    pub last_tick: Instant,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            input_mode: InputMode::Normal,
            tick_rate,
            last_tick: Instant::now(),
        }
    }

    pub async fn next_event(&mut self) -> io::Result<Option<EventType>> {
        let timeout = self
            .tick_rate
            .checked_sub(self.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                return Ok(Some(EventType::Input(key.code)));
            }
        }

        if self.last_tick.elapsed() >= self.tick_rate {
            self.last_tick = Instant::now();
            return Ok(Some(EventType::Tick));
        }

        Ok(None)
    }
}

pub fn setup_terminal() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

pub fn restore_terminal() -> io::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    Ok(())
} 