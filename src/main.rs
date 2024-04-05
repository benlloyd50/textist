use crossterm::terminal::size;
// Based on a tutorial blog post: https://archive.flenker.blog/hecto-chapter-3/
// Modified to use crossterm
use editor::Editor;

mod document;
mod editor;
mod terminal;

fn main() {
    // let term_size = size();
    // dbg!("{}", term_size.unwrap());
    Editor::default().run();
}
