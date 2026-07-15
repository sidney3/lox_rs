use std::fmt;

use itertools::Itertools;

use super::constant::Constant;
use super::instruction::Instruction;

pub struct Chunk {
  pub instructions: Vec<Instruction>,
}

impl fmt::Display for Chunk {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Instructions: [{}]", self.instructions.iter().join(", "))?;

    Ok(())
  }
}
