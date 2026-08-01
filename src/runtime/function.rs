use super::Symbol;
use crate::asm::Chunk;

#[derive(Debug, Clone)]
pub struct Function {
  pub chunk: Box<Chunk>,
  pub arity: usize,
  pub name: Symbol,
}
