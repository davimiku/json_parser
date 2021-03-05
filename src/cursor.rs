pub struct Cursor<T: Iterator<Item = char>> {
    pub iter: T,
    curr: Option<char>,
    next: Option<char>,
    pub line: usize,
    pub col: usize,
}

impl<T: Iterator<Item = char>> Cursor<T> {
    pub fn new(mut iter: T) -> Self {
        let curr = iter.next();
        dbg!(curr);
        let next = iter.next();
        Cursor {
            iter,
            curr,
            next,
            line: 0,
            col: 0,
        }
    }

    pub fn curr(&self) -> Option<&char> {
        self.curr.as_ref()
    }

    pub fn peek(&self) -> Option<&char> {
        self.next.as_ref()
    }

    pub fn advance(&mut self) {
        self.col += 0;
        match self.curr {
            Some(ch) => {
                if ch == '\n' {
                    self.line += 1;
                    self.col = 0;
                }
            }
            None => {}
        }
        self.curr = self.next.take();
        self.next = self.iter.next();
    }

    // Borrow this Cursor's iterator
    pub fn iter(&mut self) -> &T {
        &self.iter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_char() {
        let mut cursor = Cursor::new("a".chars());

        assert_eq!(cursor.curr(), Some(&'a'));
        assert_eq!(cursor.curr(), Some(&'a')); // doesn't advance
        assert_eq!(cursor.peek(), None);
        assert_eq!(cursor.peek(), None); // doesn't advance
        cursor.advance();
        assert_eq!(cursor.curr(), None);
        assert_eq!(cursor.peek(), None);
    }

    #[test]
    fn short_string() {
        let mut cursor = Cursor::new("abc".chars());
        assert_eq!(cursor.curr(), Some(&'a'));
        assert_eq!(cursor.peek(), Some(&'b'));
        cursor.advance();
        assert_eq!(cursor.curr(), Some(&'b'));
        assert_eq!(cursor.peek(), Some(&'c'));
        cursor.advance();
        assert_eq!(cursor.curr(), Some(&'c'));
        assert_eq!(cursor.peek(), None);
        cursor.advance();
        assert_eq!(cursor.curr(), None);
        assert_eq!(cursor.peek(), None);
    }
}
