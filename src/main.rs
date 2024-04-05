// Based on a tutorial blog post: https://archive.flenker.blog/hecto-chapter-3/
// Modified to use crossterm
use std::io;

use editor::Editor;

mod editor;
mod terminal;

fn main() {
    Editor::default().run();
}
