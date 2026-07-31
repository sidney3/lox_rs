use std::fmt;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd)]
pub struct Span {
  pub start: usize,
  pub end: usize,
}

impl Span {
  pub const fn new(start: usize, end: usize) -> Self {
    Self { start, end }
  }

  pub fn lexeme<'a>(&self, program: &'a str) -> &'a str {
    &program[self.start..self.end]
  }

  pub fn combine_overlapping(self, other: Span) -> Option<Self> {
    let (earlier, later) = if self.start <= other.start {
      (self, other)
    } else {
      (other, self)
    };

    if earlier.end < later.start {
      None
    } else {
      Some(Span::new(earlier.start, later.end))
    }
  }
}

impl fmt::Display for Span {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[{},{})", self.start, self.end)
  }
}
