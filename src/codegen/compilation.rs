use super::Chunk;
use super::Constant;

pub struct Compilation {
  pub chunk: Chunk,
  pub symbols: lasso::Rodeo,
  pub constants: Vec<Constant>,
}
