use super::{Function, Obj, ObjData, ObjKind, Runtime, UpValue};
use crate::gc::Ref;

#[derive(Debug)]
pub struct Closure {
  pub func: Obj<Function>,
  pub upvalues: Vec<Obj<UpValue>>,
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
