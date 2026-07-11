use std::fmt;

use crate::gc::RootedObj;

#[derive(Debug, Clone)]
pub enum Value {
  Num(f64),
  Obj(RootedObj),
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Value::Num(x) => write!(f, "{}", x),
      Value::Obj(handle) => write!(f, "{}", handle.deref()),
    }
  }
}
