use std::fmt;

use super::function::Function;

pub type LoxString = String;

#[derive(Clone, Debug)]
pub enum Constant {
  Float(f64),
  String(LoxString),
  Bool(bool),
  Func(Function),
  Nil,
}

impl fmt::Display for Constant {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Constant::Float(x) => write!(f, "{x}"),
      Constant::String(s) => write!(f, "{s}"),
      &Constant::Bool(x) => {
        write!(f, "{}", if x { "true" } else { "false" })
      }
      Constant::Nil => write!(f, "nil"),
      Constant::Func(_) => write!(f, "LoxFunc"),
    }
  }
}
