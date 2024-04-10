use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn control_held(ev_key: KeyEvent) -> bool {
    ev_key.modifiers.contains(KeyModifiers::CONTROL)
}
