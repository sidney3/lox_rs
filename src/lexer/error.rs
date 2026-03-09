use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    UnterminatedEscape,
    UnterminatedRegex,
    MalformattedRange,
    UnorderedRange(char, char),
    NoMatchingToken { line: usize },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnterminatedEscape => write!(f, "UnterminatedEscape"),
            Error::UnterminatedRegex => write!(f, "UnterminatedRegex"),
            Error::MalformattedRange => write!(f, "MalformattedRange"),
            Error::UnorderedRange(l, r) => write!(f, "Out of order range [{}, {}]", l, r),
            Error::NoMatchingToken { line } => write!(f, "No matching token on line {}", line),
        }
    }
}
