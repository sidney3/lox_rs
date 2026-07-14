use std::fmt;

#[derive(Clone)]
pub enum Constant {
  Float(f64),
  String(String),
  Bool(bool),
}

impl fmt::Display for Constant {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Constant::Float(x) => write!(f, "{x}"),
      Constant::String(s) => write!(f, "{s}"),
      &Constant::Bool(x) => {
        write!(f, "{}", if x { "true" } else { "false" })
      }
    }
  }
}
