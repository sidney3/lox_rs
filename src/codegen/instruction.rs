use std::fmt;

use strum::{Display, FromRepr};

#[derive(FromRepr)]
#[repr(u8)]
#[derive(Copy, Clone, Display)]
pub enum InstructionKind {
  Return,
  Constant,
  Add,
  Sub,
  Mult,
  Divide,
  Pop,
  Equals,
  Geq,
  Leq,
  Neq,
  Greater,
  Less,
  // TODO: remove when we get functions
  // working
  Print,
  Assert,
}

#[derive(Copy, Clone)]
pub struct Instruction {
  pub kind: InstructionKind,
  pub operand: u8,
}

impl Instruction {
  pub fn new(kind: InstructionKind) -> Self {
    Self { kind, operand: 0 }
  }
}

impl fmt::Display for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({} {})", self.kind, self.operand)
  }
}
