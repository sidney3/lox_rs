use super::ObjData;
use super::ObjKind;
use crate::gc::{Trace, Tracer};

#[derive(Debug)]
pub struct ClassDef {}

#[derive(Debug)]
pub struct ClassInstance {}

impl ClassDef {
  pub fn new() -> Self {
    Self {}
  }
}

impl ObjKind for ClassDef {
  fn embed(self) -> ObjData {
    ObjData::ClassDef(self)
  }
  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::ClassDef(class) => Some(class),
      _ => None,
    }
  }
  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::ClassDef(class) => Some(class),
      _ => None,
    }
  }
}

impl ClassInstance {
  pub fn new(_: &ClassDef) -> Self {
    Self {}
  }
}

impl Trace<ObjData> for ClassDef {
  fn trace(&self, _: &mut Tracer<ObjData>) {}
}

impl ObjKind for ClassInstance {
  fn embed(self) -> ObjData {
    ObjData::ClassInstance(self)
  }
  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::ClassInstance(class) => Some(class),
      _ => None,
    }
  }
  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::ClassInstance(class) => Some(class),
      _ => None,
    }
  }
}

impl Trace<ObjData> for ClassInstance {
  fn trace(&self, _: &mut Tracer<ObjData>) {}
}
