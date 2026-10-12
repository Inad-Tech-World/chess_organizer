//! Fixed-column helpers. TRF columns are 1-based and inclusive, counted in
//! characters (not bytes) so names with accents keep their alignment.

pub(crate) struct Line {
    chars: Vec<char>,
}

impl Line {
    pub fn new(s: &str) -> Self {
        Self {
            chars: s.chars().collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    /// Characters `from..=to`, trimmed. Missing columns read as blank.
    pub fn field(&self, from: usize, to: usize) -> String {
        let start = (from - 1).min(self.chars.len());
        let end = to.min(self.chars.len());
        self.chars[start..end]
            .iter()
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Everything from column `from` to the end of the line, trimmed.
    pub fn rest(&self, from: usize) -> String {
        self.field(from, self.chars.len().max(from))
    }

    pub fn char_at(&self, col: usize) -> char {
        self.chars.get(col - 1).copied().unwrap_or(' ')
    }
}

pub(crate) struct LineBuf {
    chars: Vec<char>,
}

impl LineBuf {
    pub fn new(code: &str) -> Self {
        let mut b = Self { chars: Vec::new() };
        b.put_left(1, 3, code);
        b
    }

    fn ensure(&mut self, to: usize) {
        if self.chars.len() < to {
            self.chars.resize(to, ' ');
        }
    }

    /// Writes `s` left-aligned in `from..=to`, truncating if needed.
    pub fn put_left(&mut self, from: usize, to: usize, s: &str) {
        self.ensure(to);
        for (i, c) in s.chars().take(to - from + 1).enumerate() {
            self.chars[from - 1 + i] = c;
        }
    }

    /// Writes `s` right-aligned in `from..=to`, keeping its rightmost characters.
    pub fn put_right(&mut self, from: usize, to: usize, s: &str) {
        let width = to - from + 1;
        let chars: Vec<char> = s.chars().collect();
        let tail = &chars[chars.len().saturating_sub(width)..];
        self.ensure(to);
        let start = to - tail.len();
        for (i, c) in tail.iter().enumerate() {
            self.chars[start + i] = *c;
        }
    }

    pub fn put_char(&mut self, col: usize, c: char) {
        self.ensure(col);
        self.chars[col - 1] = c;
    }

    pub fn finish(self) -> String {
        self.chars
            .into_iter()
            .collect::<String>()
            .trim_end()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_writes_columns() {
        let mut b = LineBuf::new("001");
        b.put_right(5, 8, "12");
        b.put_left(15, 47, "Müller, Jörg");
        let s = b.finish();
        assert_eq!(s, "001   12      Müller, Jörg");
        let l = Line::new(&s);
        assert_eq!(l.field(5, 8), "12");
        assert_eq!(l.field(15, 47), "Müller, Jörg");
        assert_eq!(l.field(90, 99), "");
        assert_eq!(l.char_at(200), ' ');
    }
}
