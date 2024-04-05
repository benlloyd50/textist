use std::{cmp, env, process::exit, time::Duration};

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::disable_raw_mode,
};

use crate::document::Document;
use crate::terminal::Terminal;

const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    dirty: bool,
    terminal: Terminal,
    document: Document,
    cursor: Position,
    offset: Position,
}

pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let file_name = &args[1];
            Document::open(file_name)
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            dirty: true,
            terminal: Terminal::setup().expect("Problem initializing terminal for editor."),
            document,
            cursor: Position { x: 0, y: 0 },
            offset: Position { x: 0, y: 0 },
        }
    }
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if self.dirty {
                self.refresh_screen();
                self.draw_rows();
                self.draw_status_bar();
                self.dirty = false;
            }
            Terminal::move_cursor(&Position {
                x: self.cursor.x.saturating_sub(self.offset.x),
                y: self.cursor.y.saturating_sub(self.offset.y),
            });

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
        Terminal::move_cursor(&Position { x: 0, y: 0 });
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
            KeyCode::Up | KeyCode::Left | KeyCode::Right | KeyCode::Down => {
                self.move_cursor(ev_key.code)
            }
            _ => {}
        }

        Ok(())
    }

    fn draw_rows(&self) {
        let height = self.terminal.size.height as usize + self.offset.y;
        let width = self.terminal.size.width as usize + self.offset.x;
        for i in self.offset.y..height - 1 {
            match self.document.rows.get(i as usize) {
                Some(s) => println!("{}\r", s.render(self.offset.x, width)),
                None => println!("~\r"),
            }
        }

        if self.document.is_empty() {
            self.draw_welcome_message(height);
        }

        Terminal::move_cursor(&Position { x: 0, y: 0 });
    }

    fn draw_welcome_message(&self, height: usize) {
        let welcome_msg = format!("TextRighter -- {}", EDITOR_VERSION);
        let width = self.terminal.size.width;
        let start_left = (width / 2) - (welcome_msg.len() as u16 / 2);
        Terminal::move_cursor(&Position {
            x: start_left as usize,
            y: height as usize / 3,
        });
        println!("{}", welcome_msg);
        Terminal::move_cursor(&Position { x: 0, y: 0 });
    }

    fn move_cursor(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => self.cursor.y = self.cursor.y.saturating_sub(1),
            KeyCode::Down => self.cursor.y = self.cursor.y.saturating_add(1),
            KeyCode::Left => self.cursor.x = self.cursor.x.saturating_sub(1),
            KeyCode::Right => self.cursor.x = self.cursor.x.saturating_add(1),
            _ => unreachable!("only entered if keycode was an arrow"),
        };
        // TODO: move most of what is under here into their functions, (maybe configurable as
        // well), this `move_cursor` should only move the cursor

        // stop cursor before end of file
        self.cursor.y = cmp::min(self.cursor.y, self.document.len() - 1);

        let max_x = match self.document.rows.get(self.cursor.y) {
            Some(row) => row.len(),
            None => 0,
        };
        self.cursor.x = cmp::min(self.cursor.x, max_x);

        // pull viewing window when on the edge of it
        if self.cursor.x > self.offset.x + self.terminal.size.width as usize - 1 {
            self.offset.x += 1;
        } else if self.cursor.x < self.offset.x {
            self.offset.x = self.cursor.x;
        }

        if self.cursor.y > self.offset.y + self.terminal.size.height as usize - 1 {
            self.offset.y += 1;
        } else if self.cursor.y < self.offset.y {
            self.offset.y = self.cursor.y;
        }
    }

    fn draw_status_bar(&self) {
        Terminal::move_cursor(&Position {
            x: 0,
            y: self.terminal.size.height as usize,
        });

        let status_notes = vec![
            "src/filename.txt".to_string(),
            "INSERT".to_string(),
            "22, 3".to_string(),
        ];
        let fmt_status = equispace_words(self.terminal.size.width.into(), &status_notes);
        print!("{}", fmt_status);
    }
}

// BUG: last word is not being right aligned
fn equispace_words(width: usize, words: &[String]) -> String {
    let total_word_len = words.iter().fold(0, |mut acc, s| {
        acc += s.len();
        acc
    });

    if total_word_len > width {
        return "WORDS TOO BIG FOR WIDTH NO STATUS BAR FOR YOU ;(".to_string();
    }

    let space_remaining = width - total_word_len;
    let space_between = space_remaining / (words.len() - 1);
    let mut output = "".to_string();
    for (idx, word) in words.iter().enumerate() {
        output += word;

        if idx < words.len() - 1 {
            output += &" ".repeat(space_between);
        }
    }
    output
}
