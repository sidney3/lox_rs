use super::{Chunk, LoxString};

pub struct Function {
  pub chunk: Chunk,
  pub arity: usize,
  pub name: LoxString,
}
