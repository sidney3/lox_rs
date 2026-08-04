use std::collections::HashMap;

use lasso::Spur;

use super::Obj;
use super::ObjData;
use super::ObjKind;
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

  pub fn borrow(&self, h: Handle) -> gc::Ref<'_, ObjData> {
    self.heap.borrow(h)
  }

  pub fn borrow_mut(&self, h: Handle) -> gc::RefMut<'_, ObjData> {
    self.heap.borrow_mut(h)
  }

  pub fn alloc(&mut self, obj: ObjData) -> Handle {
    self.heap.alloc(obj)
  }

  pub fn alloc_typed<T: ObjKind>(&mut self, obj: T) -> Obj<T> {
    Obj::<T>::downcast(self.heap.alloc(obj.embed()), self)
      .expect("We know the type of the object we just allocated")
  }

  pub fn stack_top(&self) -> usize {
    self.value_stack.len()
  }
}
