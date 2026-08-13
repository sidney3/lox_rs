use super::Trace;
use super::Tracer;
use super::header::GcHeader;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ptr::NonNull;

use super::block::GcBlock;
use super::{Ref, RefMut};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Index(usize);

#[derive(Debug)]
pub struct Handle<U> {
  index: Index, // always live
  _data: PhantomData<U>,
}

impl<U> Clone for Handle<U> {
  fn clone(&self) -> Self {
    Handle::new(self.index)
  }
}

impl<U> Copy for Handle<U> {}

impl<U> Handle<U> {
  fn new(index: Index) -> Self {
    Self {
      index,
      _data: PhantomData,
    }
  }
}

pub struct Heap<U> {
  storage: Vec<GcBlock<U>>,
  live_blocks: HashSet<Index>,
  free_blocks: Vec<Index>,
  cur_generation: usize,
}

impl<U> Heap<U> {
  pub fn new(initial_size: usize) -> Self {
    let mut heap = Self {
      storage: Vec::new(),
      live_blocks: HashSet::new(),
      free_blocks: Vec::new(),
      cur_generation: 0,
    };
    heap.grow(initial_size);
    heap
  }

  fn grow(&mut self, by: usize) {
    let old_len = self.storage.len();
    let new_indices = (0..by).map(|i| Index(old_len + i));

    self.storage.resize_with(self.storage.len() + by, || {
      GcBlock::<U>::new(self.cur_generation)
    });
    self.free_blocks.extend(new_indices);
  }

  fn capacity(&self) -> usize {
    self.free_blocks.len()
  }

  pub fn alloc(&mut self, insertee: U) -> Handle<U> {
    if self.capacity() == 0 {
      self.grow(self.storage.len());
    }

    let block = self.free_blocks.pop().expect("Just resized");

    self.storage[block.0].data.write(insertee);
    self.live_blocks.insert(block);

    Handle::<U>::new(block)
  }

  unsafe fn borrow_unchecked(&self, gc: Handle<U>) -> &U {
    unsafe {
      let block = &self.storage[gc.index.0];
      block.data.assume_init_ref()
    }
  }

  pub fn borrow<'a>(&'a self, gc: Handle<U>) -> Ref<'a, U> {
    unsafe {
      let block = &self.storage[gc.index.0];
      Ref::new(
        &block.header,
        NonNull::from_ref(block.data.assume_init_ref()),
      )
    }
  }

  pub fn borrow_mut<'a>(&'a self, gc: Handle<U>) -> RefMut<'a, U> {
    unsafe {
      let block = &self.storage[gc.index.0];
      RefMut::new(
        &block.header,
        NonNull::from_ref(block.data.assume_init_ref()),
      )
    }
  }

  pub fn collect<Root: Trace<U>>(&mut self, root: &Root)
  where
    U: Trace<U>,
  {
    self.mark(root);
    self.sweep();

    self.cur_generation += 1;
  }

  fn mark<Root: Trace<U>>(&self, root: &Root)
  where
    U: Trace<U>,
  {
    let mut tracer = Tracer::new(self, self.cur_generation);
    root.trace(self, &mut tracer);

    while let Some(next) = tracer.pop() {
      unsafe {
        self.borrow_unchecked(next).trace(self, &mut tracer);
      }
    }
  }
  fn sweep(&mut self) {
    // Ideally we would just use self.block_set, but we want to borrow from
    // self while mutating these...
    let mut next_live_blocks = std::mem::take(&mut self.live_blocks);
    let mut next_free_blocks = std::mem::take(&mut self.free_blocks);

    let removed =
      next_live_blocks.extract_if(|i| !self.index_header(*i).valid_until() > self.cur_generation);

    next_free_blocks.extend(removed);

    self.live_blocks = next_live_blocks;
    self.free_blocks = next_free_blocks;
  }

  fn index_header(&self, index: Index) -> &GcHeader {
    &self.storage[index.0].header
  }

  pub(super) fn header(&self, handle: Handle<U>) -> &GcHeader {
    self.index_header(handle.index)
  }

  #[cfg(test)]
  fn is_live(&self, index: Index) -> bool {
    self.index_header(index).valid_until() == self.cur_generation
  }
}

impl<U: Trace<U>> Trace<U> for Handle<U> {
  fn trace(&self, heap: &Heap<U>, t: &mut Tracer<U>) {
    t.mark(*self);
    unsafe {
      heap.borrow_unchecked(*self).trace(heap, t);
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  struct Obj {
    reachable: Vec<Handle<Obj>>,
  }

  impl Obj {
    pub fn new() -> Self {
      Self {
        reachable: Vec::new(),
      }
    }

    pub fn with_children(children: Vec<Handle<Obj>>) -> Self {
      Self {
        reachable: children,
      }
    }
  }

  impl Trace<Obj> for Obj {
    fn trace(&self, _: &Heap<Obj>, t: &mut Tracer<Obj>) {
      for next in self.reachable.iter().cloned() {
        t.mark(next);
      }
    }
  }

  #[test]
  fn test_simple() {
    let mut heap = Heap::new(1);

    let unreachable = heap.alloc(Obj::new());
    let unreachable_parent = heap.alloc(Obj::with_children(vec![unreachable]));

    let reachable = heap.alloc(Obj::new());

    assert!(heap.is_live(unreachable.index));
    assert!(heap.is_live(unreachable_parent.index));
    assert!(heap.is_live(reachable.index));

    heap.collect(&reachable);

    for i in (0..heap.capacity()).map(Index) {
      if i == reachable.index {
        assert!(heap.is_live(i));
      } else {
        assert!(!heap.is_live(i));
      }
    }
  }
}
