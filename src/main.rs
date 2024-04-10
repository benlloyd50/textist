// Based on a tutorial blog post: https://archive.flenker.blog/hecto-chapter-3/
// Modified to use crossterm
use editor::Editor;

mod document;
mod editor;
mod keybinds;
mod modal;
mod status_message;
mod terminal;

fn main() {
    Editor::default().run();
}
