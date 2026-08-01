use super::{Function, Obj, ObjData, ObjKind, Runtime};
use crate::gc::Ref;

#[derive(Debug)]
pub struct Closure {
  pub func: Obj<Function>,
}

impl Closure {
  pub fn borrow_func<'a>(&self, vm: &'a Runtime) -> Ref<'a, Function> {
    self.func.borrow(vm)
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
