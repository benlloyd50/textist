use std::{cmp, fs};

#[derive(Default)]
pub struct Document {
    pub rows: Vec<Row>,
}

impl Document {
    // Opens the file, file_name, or if that is not possible will open an empty document
    pub fn open(file_name: &str) -> Self {
        let contents = match fs::read_to_string(file_name) {
            Ok(c) => c,
            Err(_) => return Document::default(),
        };

        let mut rows = vec![];
        for line in contents.lines() {
            rows.push(line.into());
        }

        Self { rows }
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
        self.string.get(start..end).unwrap_or_default().to_string()
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
