use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn control_and(char: char, ev_key: KeyEvent) -> bool {
    matches!(ev_key.code, KeyCode::Char(c) if c == char)
        && ev_key.modifiers.contains(KeyModifiers::CONTROL)
}
