use std::time::Duration;

use anyhow::Result;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};

pub enum KeyCommand {
    Quit,
    TogglePause,
    None,
    Unknown,
}

impl KeyCommand {
    pub fn read(timeout: &Duration) -> Result<Self> {
        if poll(*timeout)? {
            return Ok(read()?.into());
        }

        Ok(Self::None)
    }
}

impl From<Event> for KeyCommand {
    fn from(value: Event) -> Self {
        match value {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if c == 'q' {
                    Self::Quit
                } else if c == 'p' {
                    Self::TogglePause
                } else {
                    Self::Unknown
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => Self::Quit,
            _ => Self::None,
        }
    }
}

pub fn debounce() -> Result<()> {
    loop {
        if poll(Duration::from_millis(50))? {
            let _ = read()?;
            continue;
        };

        break;
    }

    Ok(())
}

pub fn is_stdin_waiting(timeout: Duration) -> bool {
    crossterm::event::poll(timeout).expect("should be able to poll stdin")
}
