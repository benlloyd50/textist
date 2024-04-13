use std::{cmp, env, fmt::Display, process::exit, time::Duration};

use chrono::Local;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind},
    style::Stylize,
    terminal::disable_raw_mode,
};

use crate::{
    document::Document,
    modal::{Direction, InputAction, InputMode, ModalInputter, NormalInput, VerticalDirection},
};
use crate::{status_message::StatusMessage, terminal::Terminal};

const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    dirty: bool,

    terminal: Terminal,
    cursor: Position,

    document: Document,
    offset: Position,

    input: ModalInputter,
    yanked: String,

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
            status_message: StatusMessage::new("Welcome to Textist".to_string()),
            input: ModalInputter::default(),
            yanked: "".to_string(),
        }
    }
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if self.dirty {
                self.refresh_screen();
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
                        let action = self.input.process_key_press(ev_key);
                        self.handle_action(action);
                        self.dirty = true;
                        self.pull_view_to_cursor();
                    }
                    _ => {}
                }
            } else {
                // no events found
            }
        }
    }

    fn handle_action(&mut self, action: InputAction) {
        match action {
            InputAction::NoAction => {}
            InputAction::InvalidCommand => {}
            InputAction::Save => {
                self.save_document();
            }
            InputAction::Quit => {
                self.should_quit = true;
            }
            InputAction::MoveCursor { direction, count } => self.move_cursor(direction, count),
            InputAction::InsertChar(c) => {
                self.document.insert(&self.cursor, c);
                self.move_cursor(Direction::Right, 1);
            }
            InputAction::NewLine { count } => {
                self.document.add_line(&self.cursor);
                self.move_cursor(Direction::Down, 1);
                self.cursor.x = 0;
            }
            InputAction::DeleteBehind { count } => {
                self.document.remove_behind(&mut self.cursor);
            }
            InputAction::DeleteAhead { count } => {
                self.document.remove_ahead(&mut self.cursor);
            }
            InputAction::SwitchMode(new_mode) => {
                self.input.switch(new_mode);
            }
            InputAction::SaveAndQuit => {
                self.save_document();
                self.should_quit = true;
            }
            InputAction::NewLineAndInsert(vertical_direction) => match vertical_direction {
                VerticalDirection::Up => {
                    self.document.add_line_with_spaces_to_cursor(&self.cursor);
                    self.input.switch(InputMode::Insert);
                }
                VerticalDirection::Down => {
                    self.move_cursor(Direction::Down, 1);
                    self.document.add_line_with_spaces_to_cursor(&self.cursor);
                    self.input.switch(InputMode::Insert);
                }
            },
            InputAction::CommandPrompt => {
                self.input.switch(InputMode::Command);
                self.draw_status_bar();
                let action = match self.prompt(":", None) {
                    Some(cmd_input) => self.input.evaluate_cmd_input(&cmd_input),
                    None => InputAction::NoAction,
                };
                self.handle_action(action);
                self.input.switch(InputMode::Normal(NormalInput::default()));
            }
            InputAction::PasteYanked(direction) => match direction {
                Direction::Left => {
                    self.document.insert_str(&self.cursor, &self.yanked);
                    self.move_cursor(Direction::Right, self.yanked.len());
                }
                Direction::Right => {
                    self.move_cursor(Direction::Right, 1);
                    self.document.insert_str(&self.cursor, &self.yanked);
                    self.move_cursor(Direction::Right, self.yanked.len() - 1);
                }
                _ => {}
            },
            InputAction::Delete(target) => {
                self.yanked = self.document.delete(&mut self.cursor, &target);
                self.bound_cursor_to_line();
            }
        }
    }

    fn refresh_screen(&mut self) {
        Terminal::hide_cursor();
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye :)");
            let _ = disable_raw_mode();
            exit(0);
        }

        // NOTE: draw rows assume we are drawing from 0 -> terminal width/height
        self.draw_rows();
        self.draw_status_bar();
        self.draw_status_message();
        Terminal::show_cursor();
        Terminal::flush();
    }

    fn draw_rows(&self) {
        Terminal::move_cursor(&Position { x: 0, y: 0 });
        let height = self.terminal.size.height as usize + self.offset.y;
        let width = self.terminal.size.width as usize + self.offset.x;
        // 2 spaces for the status bar height
        for i in self.offset.y..height - 2 {
            Terminal::clear_line();
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
        let welcome_msg = format!("Texist -- {}", EDITOR_VERSION);
        let width = self.terminal.size.width;
        let start_left = (width / 2) - (welcome_msg.len() as u16 / 2);
        Terminal::move_cursor(&Position {
            x: start_left as usize,
            y: height as usize / 3,
        });
        println!("{}", welcome_msg);
        Terminal::move_cursor(&Position { x: 0, y: 0 });
    }

    fn move_cursor(&mut self, direction: Direction, distance: usize) {
        match direction {
            Direction::Up => self.cursor.y = self.cursor.y.saturating_sub(distance),
            Direction::Down => self.cursor.y = self.cursor.y.saturating_add(distance),
            Direction::Left => self.cursor.x = self.cursor.x.saturating_sub(distance),
            Direction::Right => self.cursor.x = self.cursor.x.saturating_add(distance),
        };

        // stop cursor before end of file
        self.bound_cursor_to_line();
    }

    // Pulls the viewport (offset) to make the cursor be in it
    fn pull_view_to_cursor(&mut self) {
        if self.cursor.x > self.offset.x + self.terminal.size.width as usize - 1 {
            self.offset.x = self
                .cursor
                .x
                .saturating_sub(self.terminal.size.width as usize)
                + 1;
        } else if self.cursor.x < self.offset.x {
            self.offset.x = self.cursor.x;
        }

        if self.cursor.y > self.offset.y + self.terminal.size.height as usize - 3 {
            self.offset.y = self
                .cursor
                .y
                .saturating_sub(self.terminal.size.height as usize)
                + 3;
        } else if self.cursor.y < self.offset.y {
            self.offset.y = self.cursor.y;
        }
    }

    fn draw_status_bar(&self) {
        Terminal::move_cursor(&Position {
            x: 0,
            y: self.terminal.size.height as usize - 2,
        });
        Terminal::clear_line();

        // config: status bar items
        let cursor_pos = self.cursor.file_position();
        let mode_text = self.input.to_string();
        let status_notes: Vec<&str> = vec![&self.document.file_name, &mode_text, &cursor_pos];
        let status_formatted = equispace_words(self.terminal.size.width.into(), &status_notes);

        // config: status bar color
        print!("{}", status_formatted.white().on_dark_blue());
    }

    // draws the status message (if there is one alive)
    fn draw_status_message(&self) {
        Terminal::move_cursor(&Position {
            x: 0,
            y: self.terminal.size.height as usize - 1,
        });

        if !self.status_message.is_showing() {
            Terminal::clear_line();
            return;
        }

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
        Terminal::clear_line();
        let mut result = start_response.unwrap_or("").to_string();
        loop {
            self.status_message
                .reset(Some(format!("{}{}", prompt, result)));
            self.draw_status_message();
            Terminal::flush();

            match read() {
                Ok(ev) => match ev {
                    Event::Key(ev_key) => {
                        if !matches!(ev_key.kind, KeyEventKind::Press) {
                            continue;
                        }
                        match ev_key.code {
                            KeyCode::Left => self.cursor.x -= 1,
                            KeyCode::Right => self.cursor.x += 1,
                            KeyCode::Char(c) => result.push(c),
                            KeyCode::Enter => return Some(result),
                            KeyCode::Esc => return None,
                            KeyCode::Backspace => match result.pop() {
                                Some(_) => {}
                                None => return None,
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                },
                Err(_) => {
                    // ignore the errors lol
                }
            };
            Terminal::clear_line();
        }
    }

    fn save_document(&mut self) {
        if self.document.file_name.is_empty() {
            let name = match self.prompt("Save as: ", None) {
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
    }

    // Pulls the cursor within the bounds of the current line and the document lines count
    fn bound_cursor_to_line(&mut self) {
        self.cursor.x = cmp::min(
            self.cursor.x,
            self.document.current_row_length(&self.cursor),
        );

        self.cursor.y = cmp::min(self.cursor.y, self.document.len().saturating_sub(1));
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
