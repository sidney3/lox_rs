use super::{BoundMethod, ClassDef, ClassInstance, Closure, Function, LoxString, UpValue};
use std::fmt;

use crate::gc::{Trace, Tracer};
use crate::runtime::{ProtoValue, Runtime, RuntimeError};

#[derive(Debug)]
pub enum ObjData {
  String(LoxString),
  Func(Function),
  Closure(Closure),
  UpValue(UpValue),
  ClassDef(ClassDef),
  ClassInstance(ClassInstance),
  BoundMethod(BoundMethod),
}

impl Trace<ObjData> for ObjData {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    match self {
      ObjData::String(s) => s.trace(tracer),
      ObjData::Func(function) => function.trace(tracer),
      ObjData::Closure(closure) => closure.trace(tracer),
      ObjData::UpValue(upval) => upval.trace(tracer),
      ObjData::ClassDef(class) => class.trace(tracer),
      ObjData::ClassInstance(class) => class.trace(tracer),
      ObjData::BoundMethod(method) => method.trace(tracer),
    }
  }
}

impl ObjData {
  pub fn add(&self, rhs: &ObjData, _: &Runtime) -> Result<ProtoValue, RuntimeError> {
    match (self, rhs) {
      (ObjData::String(lhs), ObjData::String(rhs)) => {
        Ok(ProtoValue::Obj(ObjData::String(lhs.clone() + rhs)))
      }

      // NOTE: we distinctly don't handle ValueObject here.
      // There are too many possibilities to handle. Instead, we
      // "flatten" every value.
      _ => Err(RuntimeError::new("Unsupported binary operation")),
    }
  }

  pub fn equals(&self, rhs: &Self, rt: &Runtime) -> Result<bool, RuntimeError> {
    match (self, rhs) {
      (Self::String(s1), Self::String(s2)) => Ok(s1 == s2),
      (Self::Func(_), Self::Func(_)) => Ok(false),
      (Self::ClassInstance(lhs_inst), Self::ClassInstance(rhs_inst)) => {
        lhs_inst.equals(rt, rhs_inst)
      }
      _ => Err(RuntimeError::from_str(format!(
        "TypeError: {} == {} is not defined",
        self, rhs
      ))),
    }
  }

  pub fn typename(&self) -> &'static str {
    match self {
      Self::String(_) => "LoxString",
      Self::Func(_) => "Function",
      Self::Closure(_) => "Closure",
      Self::UpValue(_) => "UpValue",
      Self::ClassDef(_) => "ClassDef",
      Self::ClassInstance(_) => "ClassInstance",
      Self::BoundMethod(_) => "BoundMethod",
    }
  }
}

impl fmt::Display for ObjData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ObjData::String(s) => write!(f, "{s}"),
      ObjData::Func(_func) => write!(f, "LoxFunc"),
      ObjData::Closure(_closure) => write!(f, "LoxClosure"),
      ObjData::UpValue(up) => match up {
        UpValue::Open { absolute_stack_pos } => write!(f, "Open UpValue --> {absolute_stack_pos}"),
        UpValue::Closed(val) => write!(f, "Closed UpValue --> {:?}", val),
      },
      ObjData::ClassDef(_) => write!(f, "ClassDef"),
      ObjData::ClassInstance(_) => write!(f, "ClassInstance"),
      ObjData::BoundMethod(_) => write!(f, "BoundMethod"),
    }
  }
}
