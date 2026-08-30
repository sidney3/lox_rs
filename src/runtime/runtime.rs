use std::collections::HashMap;
use std::collections::HashSet;

use super::value::Value;
use super::{CallFrame, Root};
use crate::gc::Ref;
use crate::gc::{self, AllocFailure, Trace, Tracer};
use crate::obj::Function;
use crate::obj::{Closure, Obj, ObjData, ObjKind};
use crate::runtime::Callee;
use lasso::Key;
use log::debug;

pub type Heap = gc::Heap<ObjData>;
pub type Handle = gc::Handle<ObjData>;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash, Eq)]
pub struct Symbol(lasso::Spur);

impl Symbol {
  pub fn try_from_usize(x: usize) -> Option<Self> {
    lasso::Spur::try_from_usize(x).map(|s| Symbol(s))
  }
  pub fn into_usize(&self) -> usize {
    self.0.into_usize()
  }
}

type Stack = Vec<Value>;
type Globals = HashMap<Symbol, Value>;

#[derive(Copy, Clone)]
pub struct FrameIndex(usize);

pub struct Runtime {
  pub value_stack: Stack,
  pub call_frames: Vec<CallFrame>,
  pub heap: Heap,
  symbols: lasso::Rodeo,
  pub globals: Globals,
  // TODO: this can be a linked list, and the
  // handle can live in the root
  roots: HashSet<Handle>,
  init_sym: Symbol,
}

pub struct GcRoot<'a, 'b, 'c, 'd> {
  stack: &'a Stack,
  globals: &'b Globals,
  frames: &'c Vec<CallFrame>,
  roots: &'d HashSet<Handle>,
}

impl<'a, 'b, 'c, 'd> GcRoot<'a, 'b, 'c, 'd> {
  pub fn new(
    stack: &'a Stack,
    globals: &'b Globals,
    frames: &'c Vec<CallFrame>,
    roots: &'d HashSet<Handle>,
  ) -> Self {
    Self {
      stack,
      globals,
      frames,
      roots,
    }
  }
}

impl Trace<ObjData> for GcRoot<'_, '_, '_, '_> {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    for val in self.stack {
      val.trace(tracer);
    }
    for val in self.globals.values() {
      val.trace(tracer);
    }

    for frame in self.frames {
      frame.trace(tracer);
    }

    for root in self.roots {
      root.trace(tracer);
    }
  }
}

impl Runtime {
  pub fn new() -> Self {
    let mut symbols = lasso::Rodeo::new();
    let init_sym = Symbol(symbols.get_or_intern_static("init"));
    Self {
      value_stack: Vec::new(),
      call_frames: Vec::new(),
      heap: Heap::new(gc::HeapConfig {
        initial_size: 4096,
        gc_growth_factor: 2,
      }),
      roots: HashSet::new(),
      symbols,
      init_sym,
      globals: HashMap::new(),
    }
  }

  pub unsafe fn borrow_unchecked<'a>(&'a self, h: Handle) -> gc::UncheckedRef<'a, ObjData> {
    unsafe { self.heap.borrow_unchecked(h) }
  }
  pub fn borrow(&self, h: Handle) -> gc::Ref<'_, ObjData> {
    self.heap.borrow(h)
  }

  pub fn resolve_sym(&self, sym: Symbol) -> &'_ str {
    self.symbols.resolve(&sym.0)
  }

  pub fn get_or_intern_sym_str(&mut self, sym_str: &str) -> Symbol {
    Symbol(self.symbols.get_or_intern(sym_str))
  }

  pub fn frame(&self, idx: FrameIndex) -> &CallFrame {
    &self.call_frames[idx.0]
  }
  pub fn frame_mut(&mut self, idx: FrameIndex) -> &mut CallFrame {
    &mut self.call_frames[idx.0]
  }

  pub fn init_sym(&self) -> Symbol {
    self.init_sym
  }

  pub fn alloc_frame(&mut self, callee: Callee, bp: usize) -> FrameIndex {
    let constants = callee.as_func(self).constants.clone();
    self.call_frames.push(CallFrame {
      callee,
      ip: 0,
      base: bp,
      constants,
    });

    FrameIndex(self.call_frames.len() - 1)
  }
  pub fn borrow_mut(&self, h: Handle) -> gc::RefMut<'_, ObjData> {
    self.heap.borrow_mut(h)
  }

  pub fn closure_func(&self, closure: &Obj<Closure>) -> Ref<'_, Function> {
    let c = closure.borrow(self);
    c.func.borrow(self)
  }

  fn collect(&mut self) {
    self.heap.collect(&GcRoot::new(
      &self.value_stack,
      &self.globals,
      &self.call_frames,
      &self.roots,
    ));
  }

  pub fn alloc(&mut self, obj: ObjData) -> Handle {
    if cfg!(feature = "stress_test_gc") {
      self.collect();
    }

    let typename = obj.typename();
    match self.heap.alloc(obj) {
      Ok(handle) => {
        debug!("Allocated {} at {:?}", typename, handle);
        handle
      }
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

  pub fn deroot<T: ObjKind>(&mut self, root: Root<T>) {
    self.roots.remove(&root.as_obj().as_handle());
    std::mem::forget(root);
  }
  pub fn root<T: ObjKind>(&mut self, obj: Obj<T>) -> Root<T> {
    self.roots.insert(obj.as_handle());
    Root::new(obj)
  }

  pub fn stack_top(&self) -> usize {
    self.value_stack.len()
  }
}
