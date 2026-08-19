use super::{Function, Obj, ObjData, ObjKind, UpValue};
use crate::gc::{Ref, Trace, Tracer};
use crate::runtime::Runtime;

#[derive(Debug)]
pub struct Closure {
  pub func: Obj<Function>,
  pub upvalues: Vec<Obj<UpValue>>,
}

impl Trace<ObjData> for Closure {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    tracer.mark(self.func.as_handle());

    for upval in &self.upvalues {
      tracer.mark(upval.as_handle());
    }
  }
}

impl Closure {
  pub fn borrow_func<'a>(&self, vm: &'a Runtime) -> Ref<'a, Function> {
    self.func.borrow(vm)
  }
}

impl Obj<Closure> {
  pub fn func<'a>(&self, vm: &'a Runtime) -> Ref<'a, Function> {
    self.borrow(vm).func.borrow(vm)
  }
}

impl ObjKind for Closure {
  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::Closure(s) => Some(s),
      _ => None,
    }
  }

  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::Closure(s) => Some(s),
      _ => None,
    }
  }

  fn embed(self) -> ObjData {
    ObjData::Closure(self)
  }
}
