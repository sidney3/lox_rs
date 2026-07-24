use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Index out of range: '{0}")]
  IndexOutOfRange(usize),

  #[error("Naked break")]
  NakedBreak,
}

pub type Result<T> = std::result::Result<T, Error>;
