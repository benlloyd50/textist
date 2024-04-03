// Based on a tutorial blog post: https://archive.flenker.blog/hecto-chapter-3/
// Modified to use crossterm
use std::io;

use crossterm::{
    cursor::MoveTo, execute, terminal::{Clear, ClearType, SetSize}
};
use editor::Editor;

mod editor;

const TERM_COLS: u16 = 120;
const TERM_ROWS: u16 = 40;

fn main() {
    let _ = execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0), SetSize(TERM_COLS, TERM_ROWS));
    Editor::default().run();
}
