use super::{Chunk, LoxString, Symbol};

#[derive(Debug, Clone)]
pub struct Function {
  pub chunk: Box<Chunk>,
  pub arity: usize,
  pub name: Symbol,
}
