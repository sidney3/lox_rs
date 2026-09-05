use super::Handle;
use super::Heap;

pub struct Tracer<'gc, U> {
  heap: &'gc Heap<U>,

  // TODO: take this from the heap. Needless
  // optimization right now...
  grey: Vec<Handle<U>>,
}

impl<'gc, U> Tracer<'gc, U> {
  pub fn new(heap: &'gc Heap<U>) -> Self {
    Self {
      heap,
      grey: Vec::new(),
    }
  }

  pub fn mark(&mut self, elt: Handle<U>) {
    if self.heap.header(elt).mark() {
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
  fn trace(&self, tracer: &mut Tracer<U>);
}
