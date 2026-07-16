use std::collections::HashMap;

use lasso::Spur;

use super::module_execution::ModuleExecution;
use super::obj::ObjData;
use super::runtime_error::RuntimeError;
use super::value::Value;
use crate::codegen::{Chunk, Compilation, Constant};
use crate::gc;

pub type Heap = gc::Heap<ObjData>;
pub type Handle = gc::Handle<ObjData>;
pub type Root<'a> = gc::Root<'a, ObjData>;

pub struct Vm {
  pub stack: Vec<Value>,
  pub heap: Heap,
  pub symbols: lasso::Rodeo,
  pub globals: HashMap<Spur, Value>,
}

impl Vm {
  pub fn new() -> Self {
    Self {
      stack: Vec::new(),
      heap: Heap::new(),
      symbols: lasso::Rodeo::new(),
      globals: HashMap::new(),
    }
  }

  pub fn run(&mut self, compilation: &Compilation) -> Result<(), RuntimeError> {
    ModuleExecution::new(self, compilation).execute()
  }

  pub fn obj(&self, h: Handle) -> gc::Ref<'_, ObjData> {
    self.heap.borrow(h)
  }

  pub fn load_const(&mut self, x: &Constant) -> Value {
    match x {
      &Constant::Float(f) => Value::Num(f),
      Constant::String(s) => self.alloc(ObjData::String(s.clone())),
      &Constant::Bool(b) => Value::Bool(b),
    }
  }

  pub fn alloc(&mut self, obj: ObjData) -> Value {
    Value::Obj(self.heap.alloc(obj))
  }
}
