use std::fmt;

use super::RuntimeError;
use super::value::Num;

type LoxString = String;

#[derive(Debug)]
pub enum ObjData {
  String(LoxString),
}

impl ObjData {
  pub fn add_right(&self, x: Num) -> Result<ObjData, RuntimeError> {
    match self {
      ObjData::String(s) => Ok(ObjData::String(format!("{s}{x}"))),
    }
  }

  pub fn add_left(&self, x: Num) -> Result<ObjData, RuntimeError> {
    match self {
      ObjData::String(s) => Ok(ObjData::String(format!("{x}{s}"))),
    }
  }
}

impl fmt::Display for ObjData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ObjData::String(s) => write!(f, "{s}"),
    }
  }
}
