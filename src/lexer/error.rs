use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unterminated escape character: '{0}")]
    UnterminatedEscape(String),

    #[error("Unterminated regex: '{0}")]
    UnterminatedRegex(String),

    #[error("Malformatted range: '{0}")]
    MalformattedRange(String),

    #[error("Unordered range: '{0}'-'{1}'")]
    UnorderedRange(char, char),

    #[error("At line {line_number}, no matching token for {line}")]
    NoMatchingToken { line: String, line_number: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
