use super::module_execution::ModuleExecution;
use super::runtime_error::RuntimeError;
use super::value::Value;
use crate::codegen::{Chunk, Constant};
use crate::gc::{Heap, ObjData};

pub struct Vm {
  pub stack: Vec<Value>,
  pub heap: Heap,
}

impl Vm {
  pub fn new() -> Self {
    Self {
      stack: Vec::new(),
      heap: Heap::new(),
    }
  }

  pub fn run(&mut self, module: &Chunk) -> Result<(), RuntimeError> {
    ModuleExecution::new(self, module).execute()
  }

  pub fn load_const(&mut self, x: &Constant) -> Value {
    match x {
      Constant::Float(f) => Value::Num(*f),
      Constant::String(s) => Value::Obj(self.heap.alloc(ObjData::String(s.to_string())).root()),
    }
  }
}
