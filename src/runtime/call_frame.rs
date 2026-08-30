use super::Value;
use crate::gc::{Ref, Trace, Tracer};
use crate::obj::{BoundMethod, ClassDef, Closure, Function, Obj, ObjData};
use crate::runtime::Runtime;

pub enum Callee {
  Class(Obj<BoundMethod>),
  Method(Obj<BoundMethod>),
  Func(Obj<Closure>),
}

impl Callee {
  pub fn as_closure<'a>(&self, rt: &'a Runtime) -> Ref<'a, Closure> {
    match self {
      Self::Class(ctor) => ctor.borrow(rt).closure(),
      Self::Method(m) => m.borrow(rt).closure(),
      Self::Func(f) => *f,
    }
    .borrow(rt)
  }
  pub fn as_func<'a>(&self, rt: &'a Runtime) -> Ref<'a, Function> {
    self.as_closure(rt).borrow_func(rt)
  }
}

pub struct CallFrame {
  pub callee: Callee,
  pub ip: usize,
  pub base: usize,
  pub constants: Vec<Value>,
}

impl Trace<ObjData> for CallFrame {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    match self.callee {
      Callee::Class(ctor) => ctor.trace(tracer),
      Callee::Func(f) => f.trace(tracer),
      Callee::Method(m) => m.trace(tracer),
    }

    for constant in &self.constants {
      constant.trace(tracer);
    }
  }
}

impl CallFrame {
  pub fn jmp(&mut self, to: usize) {
    self.ip = to;
  }
}
