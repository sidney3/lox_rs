use super::Closure;
use super::Instance;
use super::Obj;
use super::ObjData;
use crate::gc::{Trace, Tracer};
use lox_derive::LoxObj;

#[derive(Debug, LoxObj)]
pub struct BoundMethod {
  receiver: Obj<Instance>,
  method: Obj<Closure>,
}

impl BoundMethod {
  pub fn new(receiver: Obj<Instance>, method: Obj<Closure>) -> Self {
    Self { receiver, method }
  }
  pub fn closure(&self) -> Obj<Closure> {
    self.method
  }

  pub fn receiver(&self) -> Obj<Instance> {
    self.receiver
  }
}

impl Trace<ObjData> for BoundMethod {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    self.receiver.trace(tracer);
    self.method.trace(tracer);
  }
}
