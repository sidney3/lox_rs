use crate::asm;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  AssemblerError(#[from] asm::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
