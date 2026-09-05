use crate::gc::{Trace, Tracer};
use crate::runtime::Value;
use lox_derive::LoxObj;

use super::ObjData;

#[derive(LoxObj, Debug)]
pub enum UpValue {
  Open { absolute_stack_pos: usize },
  Closed(Value),
}

impl Trace<ObjData> for UpValue {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    match self {
      Self::Closed(val) => val.trace(tracer),
      _ => {}
    }
  }
}
