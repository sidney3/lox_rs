use crate::gc::{Trace, Tracer};
use crate::runtime::Value;

use super::ObjData;
use super::ObjKind;

#[derive(Debug)]
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

impl ObjKind for UpValue {
  fn embed(self) -> ObjData {
    ObjData::UpValue(self)
  }

  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::UpValue(v) => Some(v),
      _ => None,
    }
  }

  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::UpValue(v) => Some(v),
      _ => None,
    }
  }
}
