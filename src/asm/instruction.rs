use std::fmt;

use strum::{Display, FromRepr};

pub type OperandType = u8;

#[derive(FromRepr)]
#[repr(u8)]
#[derive(Copy, Clone, Display, Debug)]
pub enum InstructionKind {
  Return,
  Constant,
  Callq,

  MakeClosure,
  LoadUpValue,
  SetUpValue,
  PopUpValue,

  InstantiateClass,
  LoadClassAttribute,
  SetClassAttribute,

  // peeks, doesn't pop
  JumpIfFalsePreserving,
  JumpIfTruePreserving,
  JumpIfFalse,
  Jmp,

  // Variables
  AddGlobal,
  LoadGlobal,
  SetGlobal,
  LoadLocal,
  SetLocal,

  // Binary expression
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

  // Unary expressions.
  Not,
  UnaryMinus,

  // TODO: remove when we get functions
  // working
  Print,
  Assert,
}

#[derive(Copy, Clone, Debug)]
pub struct Instruction {
  pub kind: InstructionKind,
  pub operand: OperandType,
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
