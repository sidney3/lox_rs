use log::warn;

use super::Runtime;
use crate::obj::{Obj, ObjKind};
use crate::runtime::Value;

pub struct Root<U: ObjKind>(Obj<U>);

impl<U: ObjKind> Drop for Root<U> {
  fn drop(&mut self) {
    // We need to remove the Root from the rootset
    // (and lifetimes are too painful if we allow
    // back pointers).
    if std::thread::panicking() {
      warn!("Cannot drop root. Should manually free");
    } else {
      panic!("Cannot drop root. Should manually free");
    }
  }
}

impl<U: ObjKind> Root<U> {
  pub(super) fn new(obj: Obj<U>) -> Self {
    Self(obj)
  }

  pub fn as_obj(&self) -> Obj<U> {
    self.0
  }

  pub fn as_value(&self) -> Value {
    self.as_obj().as_value()
  }

  pub fn free(self, rt: &mut Runtime) {
    rt.deroot(self)
  }
}
