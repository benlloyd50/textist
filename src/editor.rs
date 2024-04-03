use std::{io::{self, stdout, Write}, process::exit, time::Duration};

use crossterm::{cursor::MoveTo, event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, execute, terminal::{Clear, ClearType}};

pub struct Editor {
    should_quit: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            should_quit: false,
        }
    }
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(e) = execute!(io::stdout(), Clear(ClearType::All), MoveTo(0,0)) {
                panic!("{}", e);
            }
            if self.should_quit {
                println!("Quitting");
                exit(0);
            }

            if poll(Duration::from_millis(200)).unwrap() {
                let read = match read() {
                    Ok(read) => read,
                    Err(e) => panic!("{}", e),
                };
                match read {
                    Event::Key(ev_key) => { let _ = self.process_key_press(ev_key); },
                    _ => {}
                }
                let _ = stdout().flush();
            } else {
                // no events found
            }
        }
    }

    // only handles keys that are pressed this frame, NOT released
    fn process_key_press(&mut self, ev_key: KeyEvent) -> Result<(), std::io::Error> {
        if ev_key.kind != KeyEventKind::Press {
            return Ok(())
        }

        if matches!(ev_key.code, KeyCode::Char('q')) && ev_key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
        }

        match ev_key.code {
            KeyCode::Char(c) => print!("{}", c),
            _ => {}
        }

        Ok(())
    }
}