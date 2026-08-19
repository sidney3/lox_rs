use super::{ObjData, ObjKind};
use crate::gc::{Trace, Tracer};

pub type LoxString = String;

impl ObjKind for LoxString {
  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::String(s) => Some(s),
      _ => None,
    }
  }

  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::String(s) => Some(s),
      _ => None,
    }
  }

  fn embed(self) -> ObjData {
    ObjData::String(self)
  }
}

impl Trace<ObjData> for LoxString {
  fn trace(&self, _: &mut Tracer<ObjData>) {}
}
