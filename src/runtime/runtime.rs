use std::collections::HashMap;

use super::CallFrame;
use super::Obj;
use super::ObjData;
use super::ObjKind;
use super::value::Value;
use crate::gc::{self, AllocFailure, Trace, Tracer};
use crate::runtime::Closure;

pub type Heap = gc::Heap<ObjData>;
pub type Handle = gc::Handle<ObjData>;
pub type Symbol = lasso::Spur;

type Stack = Vec<Value>;
type Globals = HashMap<Symbol, Value>;

#[derive(Copy, Clone)]
pub struct FrameIndex(usize);

pub struct Runtime {
  pub value_stack: Stack,
  pub call_frames: Vec<CallFrame>,
  pub heap: Heap,
  pub symbols: lasso::Rodeo,
  pub globals: Globals,
}

pub struct GcRoot<'a, 'b, 'c> {
  stack: &'a Stack,
  globals: &'b Globals,
  frames: &'c Vec<CallFrame>,
}

impl<'a, 'b, 'c> GcRoot<'a, 'b, 'c> {
  pub fn new(stack: &'a Stack, globals: &'b Globals, frames: &'c Vec<CallFrame>) -> Self {
    Self {
      stack,
      globals,
      frames,
    }
  }
}

impl Trace<ObjData> for GcRoot<'_, '_, '_> {
  fn trace(&self, heap: &Heap, tracer: &mut Tracer<ObjData>) {
    for val in self.stack {
      val.trace(heap, tracer);
    }
    for val in self.globals.values() {
      val.trace(heap, tracer);
    }

    for frame in self.frames {
      frame.closure.trace(heap, tracer);
    }
  }
}

impl Runtime {
  pub fn new() -> Self {
    Self {
      value_stack: Vec::new(),
      call_frames: Vec::new(),
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

  pub fn frame(&self, idx: FrameIndex) -> &CallFrame {
    &self.call_frames[idx.0]
  }
  pub fn frame_mut(&mut self, idx: FrameIndex) -> &mut CallFrame {
    &mut self.call_frames[idx.0]
  }

  pub fn alloc_frame(&mut self, closure: Obj<Closure>, bp: usize) -> FrameIndex {
    let constants = closure
      .borrow(self)
      .func
      .borrow(self)
      .chunk
      .constants
      .clone();
    self.call_frames.push(CallFrame {
      closure,
      ip: 0,
      base: bp,
      constants,
    });

    FrameIndex(self.call_frames.len() - 1)
  }
  pub fn borrow_mut(&self, h: Handle) -> gc::RefMut<'_, ObjData> {
    self.heap.borrow_mut(h)
  }

  fn collect(&mut self) {
    self.heap.collect(&GcRoot::new(
      &self.value_stack,
      &self.globals,
      &self.call_frames,
    ));
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
