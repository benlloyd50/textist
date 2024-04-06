use std::time::{Duration, Instant};

use crossterm::style::Stylize;

pub struct StatusMessage {
    text: String,
    born_at: Instant,
    show_time: Duration,
}

impl StatusMessage {
    pub fn new(text: String) -> Self {
        Self {
            text,
            born_at: Instant::now(),
            show_time: Duration::new(5, 0),
        }
    }

    pub(crate) fn reset(&mut self, new_message: Option<String>) {
        if let Some(message) = new_message {
            self.text = message;
        }
        self.born_at = Instant::now();
    }

    pub(crate) fn is_showing(&self) -> bool {
        Instant::now() - self.born_at < self.show_time
    }

    pub(crate) fn render(&self, width: usize) -> String {
        let mut text = self.text.clone();
        text.truncate(width);
        format!("{}{}", text, &" ".repeat(width - text.len()))
            .black()
            .on_grey()
            .to_string()
    }
}
