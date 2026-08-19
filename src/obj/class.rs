use super::Function;
use super::ObjData;
use super::ObjKind;
use crate::gc::{Trace, Tracer};
use crate::runtime::Runtime;
use crate::runtime::Symbol;

#[derive(Debug)]
pub struct ClassDef {
  pub ident: Symbol,
}

#[derive(Debug)]
pub struct ClassInstance {
  ident: Symbol,
}

impl ClassDef {
  pub fn new(ident: Symbol) -> Self {
    Self { ident }
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
  pub fn new(def: &ClassDef) -> Self {
    Self { ident: def.ident }
  }

  pub fn equals(&self, rt: &Runtime, rhs: &ClassInstance) -> bool {
    rhs.ident == self.ident
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
