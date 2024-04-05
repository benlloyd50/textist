use std::{process::exit, time::Duration};

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::disable_raw_mode,
};

use crate::terminal::Terminal;

pub struct Editor {
    should_quit: bool,
    dirty: bool,
    terminal: Terminal,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            should_quit: false,
            dirty: true,
            terminal: Terminal::setup().expect("Problem initializing terminal for editor."),
        }
    }
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if self.dirty {
                self.refresh_screen();
                self.draw_rows();
                self.dirty = false;
            }

            if poll(Duration::from_millis(200)).unwrap() {
                let read = match read() {
                    Ok(read) => read,
                    Err(e) => panic!("{}", e),
                };
                match read {
                    Event::Key(ev_key) => {
                        let _ = self.process_key_press(ev_key);
                    }
                    _ => {}
                }

                self.dirty = true;
            } else {
                // no events found
            }
            Terminal::flush();
        }
    }

    fn refresh_screen(&mut self) {
        Terminal::hide_cursor();
        Terminal::clear_screen();
        Terminal::move_cursor(0, 0);
        Terminal::show_cursor();

        if self.should_quit {
            println!("Goodbye :)");
            let _ = disable_raw_mode();
            exit(0);
        }
    }

    // only handles keys that are pressed this frame, NOT released
    fn process_key_press(&mut self, ev_key: KeyEvent) -> Result<(), std::io::Error> {
        if ev_key.kind != KeyEventKind::Press {
            return Ok(());
        }

        if matches!(ev_key.code, KeyCode::Char('q'))
            && ev_key.modifiers.contains(KeyModifiers::CONTROL)
        {
            self.should_quit = true;
        }

        match ev_key.code {
            KeyCode::Char(c) => print!("{}", c),
            _ => {}
        }

        Ok(())
    }

    fn draw_rows(&self) {
        let height = self.terminal.size.height;
        for _ in 0..height - 1 {
            println!("~\r");
        }

        self.draw_welcome_message(height);

        Terminal::move_cursor(0, 0);
    }

    fn draw_welcome_message(&self, height: u16) {
        let welcome_msg = "TextRighter -- 0.0.1";
        let width = self.terminal.size.width;
        let start_left = (width / 2) - (welcome_msg.len() as u16 / 2);
        Terminal::move_cursor(start_left, height / 3);
        println!("{}", welcome_msg);
        Terminal::move_cursor(0, 0);
    }
}

