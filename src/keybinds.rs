use crossterm::event::{KeyEvent, KeyModifiers};

pub fn control_held(ev_key: KeyEvent) -> bool {
    ev_key.modifiers.contains(KeyModifiers::CONTROL)
}
