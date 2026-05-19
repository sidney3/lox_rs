use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Expected token")]
    ExpectedToken,

    #[error("Unrecognized token")]
    UnrecognizedToken, // A token lead you to a bad place

    #[error("Excess program")]
    ExcessProgram, // you tried to accept with >1 remaining rule

    #[error("Incomplete program")]
    IncompleteProgram, // you didn't complete the target rule
}

pub type Result<T> = std::result::Result<T, Error>;
