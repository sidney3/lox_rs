use std::fmt;
use std::ops::Deref;

use crate::gc::{self, Heap, Ref, RefMut, Trace, Tracer, UncheckedRef};
use crate::runtime::Root;

use super::Closure;
use super::Function;
use super::Handle;
use super::ProtoValue;
use super::Runtime;
use super::RuntimeError;
use super::UpValue;
use super::Value;
use std::marker::PhantomData;

type LoxString = String;

#[derive(Debug)]
pub enum ObjData {
  String(LoxString),
  Func(Function),
  Closure(Closure),
  UpValue(UpValue),
}

impl Trace<ObjData> for String {
  fn trace(&self, _: &Heap<ObjData>, _: &mut Tracer<ObjData>) {}
}

impl Trace<ObjData> for ObjData {
  fn trace(&self, heap: &Heap<ObjData>, tracer: &mut Tracer<ObjData>) {
    match self {
      ObjData::String(s) => s.trace(heap, tracer),
      ObjData::Func(function) => function.trace(heap, tracer),
      ObjData::Closure(closure) => closure.trace(heap, tracer),
      ObjData::UpValue(upval) => upval.trace(heap, tracer),
    }
  }
}

impl ObjData {
  pub fn add(&self, rhs: &ObjData, _: &Runtime) -> Result<ProtoValue, RuntimeError> {
    match (self, rhs) {
      (ObjData::String(lhs), ObjData::String(rhs)) => {
        Ok(ProtoValue::Obj(ObjData::String(lhs.clone() + rhs)))
      }

      // NOTE: we distinctly don't handle ValueObject here.
      // There are too many possibilities to handle. Instead, we
      // "flatten" every value.
      _ => Err(RuntimeError::new("Unsupported binary operation")),
    }
  }

  pub fn equals(&self, rhs: &Self) -> Result<bool, RuntimeError> {
    match (self, rhs) {
      (Self::String(s1), Self::String(s2)) => Ok(s1 == s2),
      (Self::Func(_), Self::Func(_)) => Ok(false),
      _ => Err(RuntimeError::from_str(format!(
        "Equality is not defined on {}{}",
        self, rhs
      ))),
    }
  }

  pub fn typename(&self) -> &'static str {
    match self {
      Self::String(_) => "LoxString",
      Self::Func(_) => "Function",
      Self::Closure(_) => "Closure",
      Self::UpValue(_) => "UpValue",
    }
  }
}

impl fmt::Display for ObjData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ObjData::String(s) => write!(f, "{s}"),
      ObjData::Func(_func) => write!(f, "LoxFunc"),
      ObjData::Closure(_closure) => write!(f, "LoxClosure"),
      ObjData::UpValue(up) => match up {
        UpValue::Open { absolute_stack_pos } => write!(f, "Open UpValue --> {absolute_stack_pos}"),
        UpValue::Closed(val) => write!(f, "Closed UpValue --> {:?}", val),
      },
    }
  }
}

pub trait ObjKind: Sized {
  fn project(obj: &ObjData) -> Option<&Self>;
  fn project_mut(obj: &mut ObjData) -> Option<&mut Self>;
  fn embed(self) -> ObjData;
}

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
  fn trace(&self, heap: &Heap<ObjData>, tracer: &mut Tracer<ObjData>) {
    tracer.mark(self.as_handle());
    unsafe {
      self.borrow_unchecked(heap).deref().trace(heap, tracer);
    }
  }
}

impl ObjKind for LoxString {
  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::String(s) => Some(s),
      _ => None,
    }
  }

  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::String(s) => Some(s),
      _ => None,
    }
  }

  fn embed(self) -> ObjData {
    ObjData::String(self)
  }
}
