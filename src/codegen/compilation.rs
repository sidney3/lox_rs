use super::Constant;

use super::Chunk;

pub struct Compilation {
  pub chunk: Chunk,
  pub symbols: lasso::Rodeo,
  pub constants: Vec<Constant>,
}
