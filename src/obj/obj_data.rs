use super::{BoundMethod, Class, Closure, Function, Instance, LoxString, UpValue};
use std::fmt;

use crate::gc::{Trace, Tracer};
use crate::runtime::{ProtoValue, Runtime, RuntimeError};

#[derive(Debug)]
pub enum ObjData {
  LoxString(LoxString),
  Function(Function),
  Closure(Closure),
  UpValue(UpValue),
  Class(Class),
  Instance(Instance),
  BoundMethod(BoundMethod),
}

impl Trace<ObjData> for ObjData {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    match self {
      ObjData::LoxString(s) => s.trace(tracer),
      ObjData::Function(function) => function.trace(tracer),
      ObjData::Closure(closure) => closure.trace(tracer),
      ObjData::UpValue(upval) => upval.trace(tracer),
      ObjData::Class(class) => class.trace(tracer),
      ObjData::Instance(class) => class.trace(tracer),
      ObjData::BoundMethod(method) => method.trace(tracer),
    }
  }
}

impl ObjData {
  pub fn add(&self, rhs: &ObjData, _: &Runtime) -> Result<ProtoValue, RuntimeError> {
    match (self, rhs) {
      (ObjData::LoxString(lhs), ObjData::LoxString(rhs)) => Ok(ProtoValue::Obj(
        ObjData::LoxString(LoxString::new(lhs.as_str().to_string() + rhs.as_str())),
      )),

      // NOTE: we distinctly don't handle ValueObject here.
      // There are too many possibilities to handle. Instead, we
      // "flatten" every value.
      _ => Err(RuntimeError::new("Unsupported binary operation")),
    }
  }

  pub fn equals(&self, rhs: &Self, rt: &Runtime) -> Result<bool, RuntimeError> {
    match (self, rhs) {
      (Self::LoxString(s1), Self::LoxString(s2)) => Ok(s1.as_str() == s2.as_str()),
      (Self::Function(_), Self::Function(_)) => Ok(false),
      (Self::Instance(lhs_inst), Self::Instance(rhs_inst)) => lhs_inst.equals(rt, rhs_inst),
      _ => Err(RuntimeError::from_str(format!(
        "TypeError: {} == {} is not defined",
        self, rhs
      ))),
    }
  }

  pub fn typename(&self) -> &'static str {
    match self {
      Self::LoxString(_) => "LoxString",
      Self::Function(_) => "Function",
      Self::Closure(_) => "Closure",
      Self::UpValue(_) => "UpValue",
      Self::Class(_) => "Class",
      Self::Instance(_) => "Instance",
      Self::BoundMethod(_) => "BoundMethod",
    }
  }
}

impl fmt::Display for ObjData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ObjData::LoxString(s) => write!(f, "{}", s.as_str()),
      ObjData::Function(_func) => write!(f, "LoxFunc"),
      ObjData::Closure(_closure) => write!(f, "LoxClosure"),
      ObjData::UpValue(up) => match up {
        UpValue::Open { absolute_stack_pos } => write!(f, "Open UpValue --> {absolute_stack_pos}"),
        UpValue::Closed(val) => write!(f, "Closed UpValue --> {:?}", val),
      },
      ObjData::Class(_) => write!(f, "Class"),
      ObjData::Instance(_) => write!(f, "Instance"),
      ObjData::BoundMethod(_) => write!(f, "BoundMethod"),
    }
  }
}
