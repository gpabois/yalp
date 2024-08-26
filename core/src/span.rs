use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

impl Ord for Cursor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.line < self.other.line {
            return std::cmp::Ordering::Less;
        }
        if self.line > self.other.line {
            return std::cmp::Ordering::Greater;
        }

        self.column.cmp(&other.column)
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self { line: 1, column: 0 }
    }
}

impl Add<NextLine> for Cursor {
    type Output = Self;

    fn add(mut self, rhs: NextLine) -> Self::Output {
        self += rhs;
        self
    }
}

impl Add<NextColumn> for Cursor {
    type Output = Self;

    fn add(mut self, rhs: NextColumn) -> Self::Output {
        self += rhs;
        self
    }
}

impl std::ops::AddAssign<NextLine> for Cursor {
    fn add_assign(&mut self, _: NextLine) {
        self.column = 0;
        self.line += 1;
    }
}

impl std::ops::AddAssign<NextColumn> for Cursor {
    fn add_assign(&mut self, _: NextColumn) {
        self.column += 1;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// The location of the Token in the stream.
pub struct Span {
    pub from: Cursor,
    pub to: Cursor,
}

impl From<Cursor> for Span {
    fn from(value: Cursor) -> Self {
        Self {
            from: value,
            to: value,
        }
    }
}

impl FromIterator<Span> for Span {
    fn from_iter<T: IntoIterator<Item = Span>>(iter: T) -> Self {
        let mut span = Span::default();

        for item in iter {
            if item.from < span.from {
                span.from = iter.from;
            }

            if item.to > span.to {
                span.to = iter.to;
            }
        }

        span
    }
}

impl Span {
    pub fn new(from: Cursor, to: Cursor) -> Self {
        Self { from, to }
    }
}

impl Add<NextLine> for Span {
    type Output = Span;

    fn add(self, rhs: NextLine) -> Self::Output {
        Self {
            from: self.from,
            to: self.to + rhs,
        }
    }
}

impl Add<NextColumn> for Span {
    type Output = Span;

    fn add(self, rhs: NextColumn) -> Self::Output {
        Self {
            from: self.from,
            to: self.to + rhs,
        }
    }
}

pub struct NextLine;
pub struct NextColumn;
