use super::ObjData;
use crate::gc::{Trace, Tracer};
use crate::runtime::{Runtime, RuntimeError, Value};
use lox_derive::LoxObj;

// TODO: redesign our whole GC Architecture so this can take
// &mut Runtime :D
pub type RawNativeFunction = dyn Fn(&Runtime, &[Value]) -> Result<Value, RuntimeError>;

#[derive(LoxObj)]
pub struct NativeFunction {
  name: String,
  f: Box<RawNativeFunction>,
}

impl NativeFunction {
  pub fn new(name: &str, f: Box<RawNativeFunction>) -> Self {
    Self {
      name: name.to_string(),
      f,
    }
  }

  pub fn name(&self) -> &str {
    self.name.as_str()
  }

  pub fn call(&self, rt: &Runtime, args: &[Value]) -> Result<Value, RuntimeError> {
    self.f.as_ref()(rt, args)
  }
}

impl Trace<ObjData> for NativeFunction {
  fn trace(&self, _: &mut Tracer<ObjData>) {}
}
