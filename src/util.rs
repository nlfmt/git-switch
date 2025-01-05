use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use std::time::Duration;

pub struct CircularCounter {
    pub size: usize,
    pub selected: usize,
}

impl CircularCounter {
    pub fn next(&mut self) -> usize {
        self.selected = (self.selected + 1) % self.size;
        self.selected
    }

    pub fn prev(&mut self) -> usize {
        if self.selected == 0 {
            self.selected = self.size - 1;
        } else {
            self.selected -= 1;
        }
        self.selected
    }
}

pub fn poll_keypress(timeout: Duration) -> std::io::Result<Option<KeyEvent>> {
    if event::poll(timeout)? {
        if let Event::Key(event) = event::read()? {
            if event.kind != KeyEventKind::Release {
                return Ok(Some(event));
            }
        }
    }
    Ok(None)
}
