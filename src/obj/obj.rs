use std::ops::Deref;

use super::{LoxString, ObjData, ObjKind};
use crate::gc::{Heap, Ref, RefMut, Trace, Tracer, UncheckedRef};
use crate::runtime::{Handle, Root, Runtime, Value};

use std::marker::PhantomData;

// The underlying storage is just ObjData, but we can brand
// the ObjData blobs with the type we know they hold.
#[derive(Debug, Hash, Eq, PartialEq, PartialOrd)]
pub struct Obj<T: ObjKind> {
  raw: Handle,
  _kind: PhantomData<T>,
}

impl<T: ObjKind> Clone for Obj<T> {
  fn clone(&self) -> Self {
    Self {
      raw: self.raw,
      _kind: PhantomData,
    }
  }
}

impl<T: ObjKind> Copy for Obj<T> {}

impl<T: ObjKind> Obj<T> {
  pub fn as_root(self, vm: &mut Runtime) -> Root<T> {
    vm.root(self)
  }
  pub fn downcast(raw: Handle, vm: &Runtime) -> Option<Self> {
    if T::project(vm.borrow(raw).deref()).is_some() {
      Some(Self {
        raw,
        _kind: PhantomData,
      })
    } else {
      None
    }
  }

  pub fn as_handle(self) -> Handle {
    self.raw
  }

  pub fn as_value(self) -> Value {
    Value::Obj(self.as_handle())
  }

  pub fn try_from_value(rt: &Runtime, val: Value) -> Option<Self> {
    if let Value::Obj(handle) = val {
      Self::downcast(handle, rt)
    } else {
      None
    }
  }

  pub fn borrow<'a>(&self, vm: &'a Runtime) -> Ref<'a, T> {
    Ref::map(vm.borrow(self.raw), |obj| {
      T::project(obj).expect("Objects cannot change type")
    })
  }
  pub fn borrow_mut<'a>(&self, vm: &'a Runtime) -> RefMut<'a, T> {
    RefMut::map(vm.borrow_mut(self.raw), |obj| {
      T::project(obj).expect("Objects cannot change type")
    })
  }
  pub unsafe fn borrow_unchecked<'a>(&self, heap: &'a Heap<ObjData>) -> UncheckedRef<'a, T> {
    unsafe {
      UncheckedRef::map(heap.borrow_unchecked(self.raw), |obj| {
        T::project(obj).expect("Objects cannot change type")
      })
    }
  }
}

impl<T: ObjKind + Trace<ObjData>> Trace<ObjData> for Obj<T> {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    tracer.mark(self.as_handle());
  }
}
