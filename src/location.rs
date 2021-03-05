use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl Location {
    pub fn new() -> Self {
        Location { row: 0, col: 0 }
    }

    pub fn move_right(&mut self) {
        self.col += 1;
    }

    pub fn move_down(&mut self) {
        self.row += 1;
        self.col = 0;
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line: {}, column: {}", self.row, self.col)
    }
}
