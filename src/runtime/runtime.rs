use std::collections::HashMap;

use super::Obj;
use super::ObjData;
use super::ObjKind;
use super::value::Value;
use crate::gc::{self, AllocFailure, Trace, Tracer};

pub type Heap = gc::Heap<ObjData>;
pub type Handle = gc::Handle<ObjData>;
pub type Symbol = lasso::Spur;

type Stack = Vec<Value>;
type Globals = HashMap<Symbol, Value>;

pub struct Runtime {
  pub value_stack: Stack,
  pub heap: Heap,
  pub symbols: lasso::Rodeo,
  pub globals: Globals,
}

pub struct GcRoot<'s, 'g> {
  stack: &'s Stack,
  globals: &'g Globals,
}

impl<'s, 'g> GcRoot<'s, 'g> {
  pub fn new(stack: &'s Stack, globals: &'g Globals) -> Self {
    Self { stack, globals }
  }
}

impl<'s, 'g> Trace<ObjData> for GcRoot<'s, 'g> {
  fn trace(&self, heap: &Heap, tracer: &mut Tracer<ObjData>) {
    for val in self.stack {
      val.trace(heap, tracer);
    }
    for val in self.globals.values() {
      val.trace(heap, tracer);
    }
  }
}

impl Runtime {
  pub fn new() -> Self {
    Self {
      value_stack: Vec::new(),
      heap: Heap::new(gc::HeapConfig {
        initial_size: 4096,
        gc_growth_factor: 2,
      }),
      symbols: lasso::Rodeo::new(),
      globals: HashMap::new(),
    }
  }

  pub unsafe fn borrow_unchecked<'a>(&'a self, h: Handle) -> gc::UncheckedRef<'a, ObjData> {
    unsafe { self.heap.borrow_unchecked(h) }
  }
  pub fn borrow(&self, h: Handle) -> gc::Ref<'_, ObjData> {
    self.heap.borrow(h)
  }

  pub fn borrow_mut(&self, h: Handle) -> gc::RefMut<'_, ObjData> {
    self.heap.borrow_mut(h)
  }

  fn collect(&mut self) {
    self
      .heap
      .collect(&GcRoot::new(&self.value_stack, &self.globals));
  }

  pub fn alloc(&mut self, obj: ObjData) -> Handle {
    if cfg!(feature = "stress_test_gc") {
      self.collect();
    }

    match self.heap.alloc(obj) {
      Ok(handle) => handle,
      Err(AllocFailure::NeedGc(obj)) => {
        self.collect();
        self.alloc(obj)
      }
    }
  }

  pub fn alloc_typed<T: ObjKind>(&mut self, obj: T) -> Obj<T> {
    Obj::<T>::downcast(self.alloc(obj.embed()), self)
      .expect("We know the type of the object we just allocated")
  }

  pub fn stack_top(&self) -> usize {
    self.value_stack.len()
  }
}
