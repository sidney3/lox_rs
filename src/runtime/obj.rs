use std::fmt;

use super::Function;
use super::RuntimeError;
use super::value::Num;

type LoxString = String;

#[derive(Debug)]
pub enum ObjData {
  String(LoxString),
  Func(Function),
}

impl ObjData {
  pub fn add_right(&self, rhs: Num) -> Result<ObjData, RuntimeError> {
    match self {
      ObjData::String(s) => Ok(ObjData::String(format!("{s}{rhs}"))),
      _ => Err(RuntimeError::from_str(format!(
        "Addition is not defined on {:?}",
        &self
      ))),
    }
  }

  pub fn add_left(&self, lhs: Num) -> Result<ObjData, RuntimeError> {
    match self {
      ObjData::String(s) => Ok(ObjData::String(format!("{lhs}{s}"))),
      _ => Err(RuntimeError::from_str(format!(
        "Addition is not defined on {}",
        &self
      ))),
    }
  }

  pub fn add(&self, rhs: &ObjData) -> Result<ObjData, RuntimeError> {
    match (self, rhs) {
      (ObjData::String(lhs), ObjData::String(rhs)) => Ok(ObjData::String(lhs.clone() + rhs)),
      _ => Err(RuntimeError::new("Unsupported binary operation")),
    }
  }

  pub fn equals(&self, rhs: &Self) -> Result<bool, RuntimeError> {
    match (self, rhs) {
      (Self::String(s1), Self::String(s2)) => Ok(s1 == s2),
      (Self::Func(_), Self::Func(_)) => Ok(false),
      _ => Err(RuntimeError::from_str(format!(
        "Equality is not defined on {}{}",
        self, rhs
      ))),
    }
  }
}

impl fmt::Display for ObjData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ObjData::String(s) => write!(f, "{s}"),
      ObjData::Func(_func) => write!(f, "LoxFunc"),
    }
  }
}
