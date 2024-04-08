use std::{cmp, env, fmt::Display, io, process::exit, time::Duration};

use chrono::Local;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::Stylize,
    terminal::{disable_raw_mode, Clear, ClearType},
};

use crate::{document::Document, keybinds::control_and};
use crate::{status_message::StatusMessage, terminal::Terminal};

const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    dirty: bool,

    terminal: Terminal,
    cursor: Position,

    document: Document,
    offset: Position,

    status_message: StatusMessage,
}

pub struct Position {
    pub x: usize,
    pub y: usize,
}
impl Position {
    // displays the position in the file, 1-based
    fn file_position(&self) -> String {
        format!("{:2}, {:2}", self.x + 1, self.y + 1)
    }
}

impl Display for Position {
    // the actualy position
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.x, self.y)
    }
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
            status_message: StatusMessage::new("Welcome to TextRighter".to_string()),
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
                if self.status_message.is_showing() {
                    self.draw_status_message();
                }
                self.dirty = false;
            }
            // where to draw the cursor on screen
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

        if control_and('q', ev_key) {
            self.should_quit = true;
        }
        if control_and('s', ev_key) {
            if self.document.file_name.is_empty() {
                let name = match self.promptn("Save as: ") {
                    Some(n) => n,
                    None => {
                        format!("unnamed_{:}.txt", Local::now().format("%Y%m%d%H%M"))
                    }
                };
                self.document.file_name = name;
            }

            match self.document.save() {
                Ok(_) => self
                    .status_message
                    .reset(Some(format!("{} was saved.", self.document.file_name))),
                Err(err) => {
                    self.status_message.reset(
                        format!(
                            "File {} unable to be saved: {}",
                            self.document.file_name, err
                        )
                        .into(),
                    );
                }
            }
            return Ok(());
        }

        match ev_key.code {
            KeyCode::Char(c) => {
                self.document.insert(&self.cursor, c);
                self.cursor.x = self.cursor.x.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Left | KeyCode::Right | KeyCode::Down => {
                self.move_cursor(ev_key.code)
            }
            KeyCode::Backspace => {
                self.document.remove_behind(&mut self.cursor);
            }
            KeyCode::Delete => {
                self.document.remove_ahead(&mut self.cursor);
            }
            KeyCode::Enter => {
                self.document.add_line(&self.cursor);
                self.cursor.y = self.cursor.y.saturating_add(1);
                self.cursor.x = 0;
            }
            _ => {}
        }

        Ok(())
    }

    fn draw_rows(&self) {
        let height = self.terminal.size.height as usize + self.offset.y;
        let width = self.terminal.size.width as usize + self.offset.x;
        // 2 spaces for the status bar height
        for i in self.offset.y..height - 2 {
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

        // config: let x go past the last character in the line
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

        if self.cursor.y > self.offset.y + self.terminal.size.height as usize - 3 {
            self.offset.y += 1;
        } else if self.cursor.y < self.offset.y {
            self.offset.y = self.cursor.y;
        }
    }

    fn draw_status_bar(&self) {
        Terminal::move_cursor(&Position {
            x: 0,
            y: self.terminal.size.height as usize - 2,
        });

        // config: status bar items
        let cursor_pos = self.cursor.file_position();
        let status_notes = vec![&self.document.file_name, "INSERT", &cursor_pos];
        let fmt_status = equispace_words(self.terminal.size.width.into(), &status_notes);

        // config: status bar color
        print!("{}", fmt_status.white().on_dark_blue());
    }

    fn draw_status_message(&self) {
        Terminal::move_cursor(&Position {
            x: 0,
            y: self.terminal.size.height as usize - 1,
        });

        print!(
            "{}",
            self.status_message.render(self.terminal.size.width.into())
        );
    }

    // Given a prompt asks the user for a string answer
    fn prompt(&mut self, prompt: &str, start_response: Option<&str>) -> Option<String> {
        Terminal::move_cursor(&Position {
            x: 0,
            y: self.terminal.size.height as usize - 1,
        });
        let mut result = start_response.unwrap_or("").to_string();
        loop {
            let _ = execute!(io::stdout(), Clear(ClearType::CurrentLine));
            self.status_message
                .reset(Some(format!("{}{}", prompt, result)));
            self.draw_status_message();
            Terminal::flush();

            match read() {
                Ok(ev) => match ev {
                    Event::Key(ev_key) => match ev_key.code {
                        KeyCode::Left => self.cursor.x -= 1,
                        KeyCode::Right => self.cursor.x += 1,
                        KeyCode::Char(c) => result.push(c),
                        KeyCode::Enter => return Some(result),
                        KeyCode::Esc => return None,
                        KeyCode::Backspace => {
                            let _ = result.pop();
                        }
                        _ => {}
                    },
                    _ => {}
                },
                Err(_) => {
                    // ignore the errors lol
                }
            };
        }
    }

    // helper to call prompt without a
    fn promptn(&mut self, arg: &str) -> Option<String> {
        self.prompt(arg, None)
    }
}

fn equispace_words(width: usize, words: &[&str]) -> String {
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

    // pads the text with space at the end
    if output.len() < width {
        output += &" ".repeat(width - output.len());
    }

    output
}
