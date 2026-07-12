use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Constant {
  Float(f64),
  String(String),
}

impl fmt::Display for Constant {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Constant::Float(x) => write!(f, "{x}"),
      Constant::String(s) => write!(f, "{s}"),
    }
  }
}
