use std::io::{self, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{enable_raw_mode, size, Clear, ClearType},
};

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    pub size: Size,
}

impl Terminal {
    // Initalizes the terminal (as well as stdout)
    pub(crate) fn setup() -> Result<Terminal, io::Error> {
        execute!(
            io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES),
        )?;
        enable_raw_mode()?;
        let term_size = size()?;
        Ok(Terminal {
            size: Size {
                width: term_size.0,
                height: term_size.1,
            },
        })
    }

    pub fn clear_screen() {
        if let Err(e) = execute!(io::stdout(), Clear(ClearType::All)) {
            panic!("{}", e);
        }
    }

    pub fn move_cursor(x: u16, y: u16) {
        if let Err(e) = execute!(io::stdout(), MoveTo(x, y)) {
            panic!("Panic during cursor movement. {}", e);
        }
    }

    pub fn show_cursor() {
        if let Err(e) = execute!(io::stdout(), Show) {
            panic!("{}", e);
        }
    }

    pub fn hide_cursor() {
        if let Err(e) = execute!(io::stdout(), Hide) {
            panic!("{}", e);
        }
    }

    pub fn flush() {
        let _ = io::stdout().flush();
    }
}
