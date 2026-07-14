use std::fmt;

type LoxString = String;

#[derive(Debug)]
pub enum ObjData {
  String(LoxString),
}

impl fmt::Display for ObjData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ObjData::String(s) => write!(f, "{s}"),
    }
  }
}
