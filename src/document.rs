use std::{
    cmp, fs,
    io::{Error, Write},
};

use crossterm::style::Stylize;

use crate::editor::Position;

#[derive(Default)]
pub struct Document {
    pub rows: Vec<Row>,
    pub file_name: String,
}

impl Document {
    // Opens the file, file_name, or if that is not possible will open an empty document
    pub fn open(file_name: &str) -> Self {
        let contents = match fs::read_to_string(file_name) {
            Ok(c) => c,
            Err(_) => {
                return Document {
                    file_name: file_name.to_string(),
                    rows: vec![],
                };
            }
        };

        let mut rows = vec![];
        for line in contents.lines() {
            let line = line.replace("\t", "    ").into();
            rows.push(line);
        }

        Self {
            rows,
            file_name: file_name.to_string(),
        }
    }

    pub fn save(&self) -> Result<(), Error> {
        match fs::File::create(&self.file_name) {
            Ok(mut file) => {
                for row in &self.rows {
                    file.write_all(row.string.as_bytes())?;
                    file.write_all(b"\n")?;
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        let row = match self.rows.get_mut(at.y) {
            Some(row) => row,
            None => {
                self.rows.insert(at.y, Row::default());
                // unwrap - insertion prior so it should be there
                self.rows.get_mut(at.y).unwrap()
            }
        };

        row.string.insert(at.x, c);
    }

    pub(crate) fn remove_behind(&mut self, at: &mut Position) {
        if at.x == 0 {
            if at.y == 0 {
                return;
            }
            let old_row = self.rows.remove(at.y);

            if let Some(row) = self.rows.get_mut(at.y - 1) {
                at.x = row.len();
                row.string += &old_row.string;
                at.y = at.y.saturating_sub(1);
            }
            return;
        }

        let row = self.rows.get_mut(at.y).unwrap();
        row.string.remove(at.x.saturating_sub(1));
        at.x = at.x.saturating_sub(1);
    }

    pub(crate) fn remove_ahead(&mut self, at: &mut Position) {
        let curr_row = self.rows.get(at.y).unwrap();
        let mut old_row = None;
        if at.x == curr_row.len() {
            if at.y >= self.rows.len() - 1 {
                return;
            }
            old_row = Some(self.rows.remove(at.y + 1));
        }

        let row = self.rows.get_mut(at.y).unwrap();
        if let Some(text) = old_row {
            row.string += &text.string;
            return;
        }

        let _ = row.string.remove(at.x);
    }

    pub(crate) fn add_line(&mut self, at: &Position) {
        {
            let Some(curr_row) = self.rows.get(at.y) else {
                return;
            };

            // take whatever is after cursor on the current line
            let text_after_cursor = curr_row.string.get(at.x..).unwrap();
            // add it to the next line
            self.rows.insert(
                at.y + 1,
                Row {
                    string: text_after_cursor.to_string(),
                },
            );
        }

        // remove the stuff we took off
        let Some(curr_row) = self.rows.get_mut(at.y) else {
            return;
        };
        curr_row.string.truncate(at.x);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Default)]
pub struct Row {
    string: String,
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let output = self.string.get(start..end).unwrap_or_default().to_string();

        // config: visible spaces
        replace_spaces_to_first_word(&output, &".".dim().to_string())
    }

    pub fn len(&self) -> usize {
        self.string.len()
    }
}

impl From<&str> for Row {
    fn from(string: &str) -> Self {
        Self {
            string: string.into(),
        }
    }
}

fn replace_spaces_to_first_word(text: &str, with: &str) -> String {
    let mut first_non_space: usize = 0;
    for c in text.chars() {
        if c == ' ' {
            first_non_space += 1;
        } else {
            break;
        }
    }

    if first_non_space == 0 {
        return text.to_string();
    }

    let mut output = String::new();
    for _ in 0..first_non_space {
        output += &with.dim().to_string();
    }
    output + text.get(first_non_space..).unwrap()
}
