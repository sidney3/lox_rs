use std::fmt;

use super::RuntimeError;
use super::value::Num;

type LoxString = String;

#[derive(Debug)]
pub enum ObjData {
  String(LoxString),
}

impl ObjData {
  pub fn add_right(&self, rhs: Num) -> Result<ObjData, RuntimeError> {
    match self {
      ObjData::String(s) => Ok(ObjData::String(format!("{s}{rhs}"))),
    }
  }

  pub fn add_left(&self, lhs: Num) -> Result<ObjData, RuntimeError> {
    match self {
      ObjData::String(s) => Ok(ObjData::String(format!("{lhs}{s}"))),
    }
  }

  pub fn equals(&self, rhs: &Self) -> bool {
    match (self, rhs) {
      (Self::String(s1), Self::String(s2)) => s1 == s2,
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
