use super::call_frame::CallFrame;
use std::collections::HashMap;

use lasso::Spur;

use super::module_execution::ModuleExecution;
use super::obj::ObjData;
use super::runtime_error::RuntimeError;
use super::value::Value;
use crate::asm::{Chunk, Constant};
use crate::codegen::Compilation;
use crate::gc;

pub type Heap = gc::Heap<ObjData>;
pub type Handle = gc::Handle<ObjData>;
pub type Root<'a> = gc::Root<'a, ObjData>;

pub struct Vm {
  pub value_stack: Vec<Value>,
  pub heap: Heap,
  pub symbols: lasso::Rodeo,
  pub globals: HashMap<Spur, Value>,
}

impl Vm {
  pub fn new() -> Self {
    Self {
      value_stack: Vec::new(),
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

  pub fn load_const(&mut self, x: Constant) -> Value {
    match x {
      Constant::Float(f) => Value::Num(f),
      Constant::String(s) => self.alloc(ObjData::String(s)),
      Constant::Bool(b) => Value::Bool(b),
      Constant::Nil => Value::Nil,
      Constant::Func(f) => self.alloc(ObjData::Func(f)),
    }
  }

  pub fn alloc(&mut self, obj: ObjData) -> Value {
    Value::Obj(self.heap.alloc(obj))
  }

  pub fn stack_top(&self) -> usize {
    self.value_stack.len()
  }
}
