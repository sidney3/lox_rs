use super::Handle;
use super::Heap;

pub struct Tracer<'gc, U> {
  heap: &'gc Heap<U>,

  collecting_generation: usize,

  // TODO: take this from the heap. Needless
  // optimization right now...
  grey: Vec<Handle<U>>,
}

impl<'gc, U> Tracer<'gc, U> {
  pub fn new(heap: &'gc Heap<U>, collecting_generation: usize) -> Self {
    Self {
      heap,
      collecting_generation,
      grey: Vec::new(),
    }
  }

  pub fn mark(&mut self, elt: Handle<U>) {
    if self.heap.header(elt).mark(self.collecting_generation) {
      self.grey.push(elt);
    }
  }

  pub(super) fn pop(&mut self) -> Option<Handle<U>> {
    self.grey.pop()
  }
}

// A type implementing trace should call `t.mark(handle)` for each `handle`
// directly reachable from self.
pub trait Trace<U> {
  fn trace(&self, heap: &Heap<U>, t: &mut Tracer<U>); // &self, concrete param → object-safe
}
