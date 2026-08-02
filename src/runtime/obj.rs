use std::fmt;
use std::ops::Deref;

use crate::gc::Ref;

use super::Closure;
use super::Function;
use super::Handle;
use super::ProtoValue;
use super::Runtime;
use super::RuntimeError;
use super::UpValue;
use super::ValueObject;
use super::value::Num;
use std::marker::PhantomData;

type LoxString = String;

#[derive(Debug)]
pub enum ObjData {
  String(LoxString),
  Func(Function),
  Closure(Closure),
  UpValue(UpValue),
  Value(ValueObject),
}

impl ObjData {
  pub fn add(&self, rhs: &ObjData, _: &Runtime) -> Result<ProtoValue, RuntimeError> {
    match (self, rhs) {
      (ObjData::String(lhs), ObjData::String(rhs)) => {
        Ok(ProtoValue::Obj(ObjData::String(lhs.clone() + rhs)))
      }
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
}

impl fmt::Display for ObjData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ObjData::String(s) => write!(f, "{s}"),
      ObjData::Func(_func) => write!(f, "LoxFunc"),
      ObjData::Closure(_closure) => write!(f, "LoxClosure"),
      ObjData::UpValue(up) => match up {
        UpValue::Open { absolute_stack_pos } => write!(f, "Open UpValue --> {absolute_stack_pos}"),
      },
      ObjData::Value(v) => write!(f, "{:?}", v),
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
#[derive(Debug)]
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

  pub fn borrow<'a>(&self, vm: &'a Runtime) -> Ref<'a, T> {
    Ref::map(vm.borrow(self.raw), |obj| {
      T::project(obj).expect("Objects cannot change type")
    })
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
