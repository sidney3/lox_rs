use super::emitter::Symbol;

#[derive(Debug)]
pub enum Error {
  IndexOutOfRange(usize),
}

pub type Result<T> = std::result::Result<T, Error>;
