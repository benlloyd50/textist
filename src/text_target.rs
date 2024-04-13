#[derive(Clone, Copy)]
pub enum TextTarget {
    Nothing,
    All,
    WholeRow,
    RowAfterCursor,
    UnderCursor,
    Char(char),
}
