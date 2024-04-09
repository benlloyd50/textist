use std::io::{self, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{enable_raw_mode, size, Clear, ClearType},
};

use crate::editor::Position;

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    pub size: Size,
}

impl Terminal {
    // Initalizes the terminal
    pub(crate) fn setup() -> Result<Terminal, io::Error> {
        if cfg!(unix) {
            execute!(
                io::stdout(),
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES),
            )?;
        }

        enable_raw_mode()?;
        let term_size = size()?;
        Ok(Terminal {
            size: Size {
                width: term_size.0,
                height: term_size.1,
            },
        })
    }

    // Clears the entire terminal display
    pub fn clear_screen() {
        if let Err(e) = execute!(io::stdout(), Clear(ClearType::All)) {
            panic!("{}", e);
        }
    }

    // Clears the line the cursor is currently positioned on
    pub fn clear_line() {
        if let Err(e) = execute!(io::stdout(), Clear(ClearType::CurrentLine)) {
            panic!("{}", e);
        }
    }

    pub fn move_cursor(position: &Position) {
        let x = match position.x.try_into() {
            Ok(x) => x,
            Err(_) => u16::MAX,
        };
        let y = match position.y.try_into() {
            Ok(y) => y,
            Err(_) => u16::MAX,
        };

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
