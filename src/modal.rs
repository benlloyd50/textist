use std::{fmt::Display, io};

// TODO: figure out ways to handle actions like z which require a second press t, b, z
// TODO: figure out ways to handle actions 100gg which require a number and a second press
use crossterm::{
    cursor::SetCursorStyle,
    event::{KeyCode, KeyEvent, KeyEventKind},
    execute,
};

use crate::{keybinds::control_held, text_target::TextTarget};

pub struct ModalInputter {
    mode: InputMode,
}

impl Default for ModalInputter {
    fn default() -> Self {
        Self {
            mode: Default::default(),
        }
    }
}

impl Display for ModalInputter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mode.to_string())
    }
}

pub enum InputAction {
    NoAction,       // used when key press cannot resolve into an action
    InvalidCommand, // when command mode does not produce a valid command
    Save,
    Quit,
    SaveAndQuit,
    MoveCursor { direction: Direction, count: usize },
    InsertChar(char),
    SwitchMode(InputMode),
    NewLine { count: usize },
    DeleteBehind { count: usize },
    DeleteAhead { count: usize },
    PasteYanked(Direction),
    NewLineAndInsert(VerticalDirection),
    CommandPrompt,
    Delete(TextTarget),
}

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

//NOTE: couples keycode to direction, may be difficult to implment keybinds for directions
impl From<KeyCode> for Direction {
    fn from(value: KeyCode) -> Self {
        match value {
            // Arrows
            KeyCode::Left => Direction::Left,
            KeyCode::Right => Direction::Right,
            KeyCode::Down => Direction::Down,
            KeyCode::Up => Direction::Up,
            // Vim
            KeyCode::Char('h') => Direction::Left,
            KeyCode::Char('k') => Direction::Up,
            KeyCode::Char('j') => Direction::Down,
            KeyCode::Char('l') => Direction::Right,

            _ => unreachable!("Trying to convert invalid keycode into direction"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum VerticalDirection {
    Up,
    Down,
}

pub enum InputMode {
    Normal(NormalInput),
    Insert,
    Command,
}

impl Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode = match self {
            InputMode::Normal(_) => "Normal",
            InputMode::Insert => "Insert",
            InputMode::Command => "Command",
        };
        write!(f, "{}", mode)
    }
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Normal(NormalInput::default())
    }
}

#[derive(Default, Clone, Copy)]
pub struct NormalInput {
    num_modifier: Option<usize>,
    command: Option<Command>,
    target: Option<TextTarget>,
}

impl NormalInput {
    // Appends the num to the existing num_modifer or creates it as Some(num)
    // Ignores 0 if it is the first digit
    // num MUST be a valid base 10 digit
    fn insert_num_modifier(&mut self, num: char) {
        let num: u32 = num.to_digit(10).unwrap();

        self.num_modifier = match self.num_modifier {
            Some(n) => Some(n * 10 + n),
            None => {
                if num == 0 {
                    None
                } else {
                    Some(num as usize)
                }
            }
        };
    }
}

#[derive(Clone, Copy)]
enum Command {
    SwitchInsert,
    Move(Direction),
    Quit,
    NewLineAndInsert(VerticalDirection),
    None,
    CommandInput,
    Delete,
    Paste(Direction), // No command, currently used when an unbound key is pressed when waiting on a command
}

impl ModalInputter {
    pub fn process_key_press(&mut self, ev_key: KeyEvent) -> InputAction {
        match self.mode {
            InputMode::Normal(input) => {
                let new_input = self.handle_normal_input(ev_key, input);
                match evaluate_normal_input(new_input) {
                    Some(action) => {
                        // the input buffer is reset since we are issuing the action
                        self.mode = InputMode::Normal(NormalInput::default());
                        action
                    }
                    None => {
                        // update the input buffer with the new input
                        self.mode = InputMode::Normal(new_input);
                        InputAction::NoAction
                    }
                }
            }
            InputMode::Insert => self.handle_insert_input(ev_key),
            InputMode::Command => InputAction::CommandPrompt,
        }
    }

    fn handle_normal_input(&self, ev_key: KeyEvent, input_buffer: NormalInput) -> NormalInput {
        if ev_key.kind != KeyEventKind::Press {
            return input_buffer;
        }

        let mut new_input = input_buffer.clone();
        match ev_key.code {
            KeyCode::Char(num) if num.is_digit(10) => {
                new_input.insert_num_modifier(num);
            }
            KeyCode::Char('o') => {
                new_input.command = Some(Command::NewLineAndInsert(VerticalDirection::Down));
            }
            KeyCode::Char('O') => {
                new_input.command = Some(Command::NewLineAndInsert(VerticalDirection::Up));
            }
            KeyCode::Char('d') => match new_input.command {
                Some(command) => match command {
                    Command::Delete => new_input.target = Some(TextTarget::WholeRow),
                    _ => new_input.command = Some(Command::None),
                },
                None => new_input.command = Some(Command::Delete),
            },
            KeyCode::Char('x') => {
                new_input.command = Some(Command::Delete);
                new_input.target = Some(TextTarget::UnderCursor);
            }
            KeyCode::Char('p') => new_input.command = Some(Command::Paste(Direction::Right)),
            KeyCode::Char('P') => new_input.command = Some(Command::Paste(Direction::Left)),
            KeyCode::Char('D') => {
                new_input.command = Some(Command::Delete);
                new_input.target = Some(TextTarget::RowAfterCursor);
            }
            KeyCode::Char('i') => {
                new_input.command = Some(Command::SwitchInsert);
            }
            KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Char('k') | KeyCode::Char('j') => {
                new_input.command = Some(Command::Move(Direction::from(ev_key.code)));
            }
            KeyCode::Char(':') => {
                new_input.command = Some(Command::CommandInput);
            }
            KeyCode::Char('Q') => match new_input.command {
                Some(_) => {
                    new_input.target = Some(TextTarget::Nothing);
                }
                None => {
                    new_input.command = Some(Command::None);
                }
            },
            KeyCode::Char('Z') => match new_input.command {
                Some(_) => {
                    new_input.target = Some(TextTarget::All);
                }
                None => {
                    new_input.command = Some(Command::Quit);
                }
            },
            _ => {}
        }

        new_input
    }

    fn handle_insert_input(&self, ev_key: KeyEvent) -> InputAction {
        if ev_key.kind != KeyEventKind::Press {
            return InputAction::NoAction;
        }

        match ev_key.code {
            KeyCode::Char('s') if control_held(ev_key) => InputAction::Save,
            KeyCode::Char(c) => InputAction::InsertChar(c),
            KeyCode::Esc => InputAction::SwitchMode(InputMode::Normal(NormalInput::default())),
            KeyCode::Up | KeyCode::Left | KeyCode::Right | KeyCode::Down => {
                InputAction::MoveCursor {
                    direction: ev_key.code.into(),
                    count: 1,
                }
            }
            KeyCode::Backspace => InputAction::DeleteBehind { count: 1 },
            KeyCode::Delete => InputAction::DeleteAhead { count: 1 },
            KeyCode::Enter => InputAction::NewLine { count: 1 },
            _ => InputAction::NoAction,
        }
    }

    pub(crate) fn switch(&mut self, new_mode: InputMode) {
        match new_mode {
            InputMode::Normal(_) => {
                let _ = execute!(io::stdout(), SetCursorStyle::BlinkingBlock);
            }
            InputMode::Insert => {
                let _ = execute!(io::stdout(), SetCursorStyle::BlinkingBar);
            }
            InputMode::Command => {}
        }
        self.mode = new_mode;
    }

    pub(crate) fn evaluate_cmd_input(&self, cmd_input: &str) -> InputAction {
        match cmd_input {
            "w" => InputAction::Save,
            "q" => InputAction::Quit,
            "wq" => InputAction::SaveAndQuit,
            _ => InputAction::InvalidCommand,
        }
    }
}

// Attempts to find a valid input action based on the buffered inputs
// If the input buffer is not valid, then InputAction::NoAction will be returned
// If the input buffer is not ready, i.e. waiting for a target, then None will be returned
fn evaluate_normal_input(input: NormalInput) -> Option<InputAction> {
    let command = match input.command {
        Some(c) => c,
        None => return None,
    };
    let count = match input.num_modifier {
        Some(num) => num,
        None => 1, // we always want to do the action atleast once
    };

    let action = match command {
        Command::Paste(direction) => InputAction::PasteYanked(direction),
        Command::CommandInput => InputAction::CommandPrompt,
        Command::SwitchInsert => InputAction::SwitchMode(InputMode::Insert),
        Command::Move(direction) => InputAction::MoveCursor { direction, count },
        Command::NewLineAndInsert(v_direction) => InputAction::NewLineAndInsert(v_direction),
        Command::Quit => match input.target {
            Some(target) => match target {
                TextTarget::Nothing => InputAction::Quit,
                TextTarget::All => InputAction::SaveAndQuit,
                _ => InputAction::NoAction,
            },
            None => return None,
        },
        Command::Delete => match input.target {
            Some(target) => InputAction::Delete(target),
            None => return None,
        },
        Command::None => InputAction::NoAction,
    };

    Some(action)
}
