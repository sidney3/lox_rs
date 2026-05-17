#[derive(Debug)]
pub enum Error {
    ExpectedToken,
    UnrecognizedToken, // A token lead you to a bad place
    ExcessProgram,     // you tried to accept with >1 remaining rule
    IncompleteProgram, // you didn't complete the target rule
    StackTooSmall,
}

pub type Result<T> = std::result::Result<T, Error>;
