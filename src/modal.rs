use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::keybinds::control_held;

pub enum InputAction {
    Save,
    Quit,
    ModeChange(InputMode),
    MoveCursor(Direction),
    InsertChar(char),
    NewLine,
    DeleteBehind,
    DeleteAhead,
    NoAction, // used when cannot resolve key press into an action
}

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

pub enum InputMode {
    Normal,
    Insert,
    Command,
}

pub trait ModalInput {
    fn name(&self) -> String;

    fn process_key_press(&self, ev_key: KeyEvent) -> InputAction;
}

// TODO: figure out ways to handle actions like z which require a second press t, b, z
// TODO: figure out ways to handle actions 100gg which require a number and a second press
pub struct Normal;

impl ModalInput for Normal {
    fn name(&self) -> String {
        "Normal".to_string()
    }

    fn process_key_press(&self, ev_key: KeyEvent) -> InputAction {
        if ev_key.kind != KeyEventKind::Press {
            return InputAction::NoAction;
        }

        match ev_key.code {
            KeyCode::Char('i') => InputAction::ModeChange(InputMode::Insert),
            KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Char('k') | KeyCode::Char('j') => {
                InputAction::MoveCursor(ev_key.code.into())
            }
            KeyCode::Char(':') => InputAction::ModeChange(InputMode::Command),
            _ => InputAction::NoAction,
        }
    }
}

pub struct Insert;

impl ModalInput for Insert {
    fn name(&self) -> String {
        "Insert".to_string()
    }

    fn process_key_press(&self, ev_key: KeyEvent) -> InputAction {
        if ev_key.kind != KeyEventKind::Press {
            return InputAction::NoAction;
        }

        match ev_key.code {
            KeyCode::Char('q') if control_held(ev_key) => InputAction::Quit,
            KeyCode::Char('s') if control_held(ev_key) => InputAction::Save,
            KeyCode::Char(c) => InputAction::InsertChar(c),
            KeyCode::Esc => InputAction::ModeChange(InputMode::Normal),
            KeyCode::Up | KeyCode::Left | KeyCode::Right | KeyCode::Down => {
                InputAction::MoveCursor(ev_key.code.into())
            }
            KeyCode::Backspace => InputAction::DeleteBehind,
            KeyCode::Delete => InputAction::DeleteAhead,
            KeyCode::Enter => InputAction::NewLine,
            _ => InputAction::NoAction,
        }
    }
}

pub struct Command;

impl ModalInput for Command {
    fn name(&self) -> String {
        "Command".to_string()
    }

    fn process_key_press(&self, ev_key: KeyEvent) -> InputAction {
        if matches!(ev_key.code, KeyCode::Esc) {
            InputAction::ModeChange(InputMode::Normal)
        } else {
            InputAction::NoAction
        }
    }
}
