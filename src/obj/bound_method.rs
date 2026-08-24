use super::ClassInstance;
use super::Closure;
use super::Obj;
use super::ObjData;
use super::ObjKind;
use crate::gc::{Trace, Tracer};

#[derive(Debug)]
pub struct BoundMethod {
  receiver: Obj<ClassInstance>,
  method: Obj<Closure>,
}

impl BoundMethod {
  pub fn new(receiver: Obj<ClassInstance>, method: Obj<Closure>) -> Self {
    Self { receiver, method }
  }
  pub fn closure(&self) -> Obj<Closure> {
    self.method
  }
}

impl ObjKind for BoundMethod {
  fn embed(self) -> ObjData {
    ObjData::BoundMethod(self)
  }
  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::BoundMethod(this) => Some(this),
      _ => None,
    }
  }
  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::BoundMethod(this) => Some(this),
      _ => None,
    }
  }
}

impl Trace<ObjData> for BoundMethod {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    self.receiver.trace(tracer);
    self.method.trace(tracer);
  }
}
