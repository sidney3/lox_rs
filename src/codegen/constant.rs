use std::fmt;

#[derive(Clone, Copy)]
pub enum Constant {
  Float(f64),
}

impl fmt::Display for Constant {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Constant::Float(x) => write!(f, "{x}"),
    }
  }
}
