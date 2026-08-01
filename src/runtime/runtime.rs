use std::collections::HashMap;

use lasso::Spur;

use super::obj::ObjData;
use super::value::Value;
use crate::gc;

pub type Heap = gc::Heap<ObjData>;
pub type Handle = gc::Handle<ObjData>;
pub type Root<'a> = gc::Root<'a, ObjData>;
pub type Symbol = lasso::Spur;

pub struct Runtime {
  pub value_stack: Vec<Value>,
  pub heap: Heap,
  pub symbols: lasso::Rodeo,
  pub globals: HashMap<Spur, Value>,
}

impl Runtime {
  pub fn new() -> Self {
    Self {
      value_stack: Vec::new(),
      heap: Heap::new(),
      symbols: lasso::Rodeo::new(),
      globals: HashMap::new(),
    }
  }

  pub fn obj(&self, h: Handle) -> gc::Ref<'_, ObjData> {
    self.heap.borrow(h)
  }

  pub fn alloc(&mut self, obj: ObjData) -> Value {
    Value::Obj(self.heap.alloc(obj))
  }

  pub fn stack_top(&self) -> usize {
    self.value_stack.len()
  }
}
