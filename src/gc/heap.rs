use super::Trace;
use super::Tracer;
use super::header::GcHeader;
use log::debug;
use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::NonNull;

use super::block::GcBlock;
use super::{Ref, RefMut, UncheckedRef};

// TODO: typestate to describe live and ambiguous
// indices.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Index(usize);

#[derive(Debug)]
pub struct Handle<U> {
  index: Index, // always live
  generation: usize,
  _data: PhantomData<U>,
}

impl<U> Clone for Handle<U> {
  fn clone(&self) -> Self {
    Handle::new(self.index, self.generation)
  }
}

impl<U> Copy for Handle<U> {}

impl<U> Handle<U> {
  fn new(index: Index, generation: usize) -> Self {
    Self {
      index,
      generation,
      _data: PhantomData,
    }
  }
}

pub struct HeapConfig {
  pub initial_size: usize,
  pub gc_growth_factor: usize,
}

pub struct Heap<U> {
  storage: Vec<GcBlock<U>>,
  live_blocks: HashSet<Index>,

  free_blocks: Vec<Index>,
  cur_generation: usize,
  config: HeapConfig,
  next_gc: usize,
}

pub type AllocResult<U> = std::result::Result<Handle<U>, AllocFailure<U>>;

// When allocation of an object O fails, we return
// the object back to the caller.
#[derive(Debug)]
pub enum AllocFailure<U> {
  NeedGc(U),
}

impl<U: Sized> Heap<U> {
  pub fn new(config: HeapConfig) -> Self {
    let initial_size = config.initial_size;
    assert!(config.initial_size > 0);
    let mut heap = Self {
      storage: Vec::new(),
      live_blocks: HashSet::new(),
      free_blocks: Vec::new(),
      cur_generation: 0,
      next_gc: 2 * initial_size * size_of::<U>(),
      config,
    };
    heap.grow(initial_size);
    heap
  }

  fn bytes_allocated(&self) -> usize {
    self.live_blocks.len() * size_of::<U>()
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

  pub fn alloc(&mut self, insertee: U) -> AllocResult<U>
  where
    U: Debug,
  {
    if self.bytes_allocated() > self.next_gc {
      return Err(AllocFailure::NeedGc(insertee));
    }

    if self.capacity() == 0 {
      self.grow(self.storage.len());
    }

    let block = self.free_blocks.pop().expect("Just resized");

    self.storage[block.0].data.write(insertee);
    self.storage[block.0].header = GcHeader::new(self.cur_generation);
    self.live_blocks.insert(block);

    Ok(Handle::<U>::new(block, self.cur_generation))
  }

  // Access Handle without any sort of borrow checking. Basically reserved
  // for reads within a garbage collection cycle.
  pub unsafe fn borrow_unchecked(&self, gc: Handle<U>) -> UncheckedRef<'_, U> {
    unsafe {
      let block = &self.storage[gc.index.0];
      UncheckedRef::new(
        &block.header,
        NonNull::from_ref(block.data.assume_init_ref()),
      )
    }
  }

  pub fn borrow<'a>(&'a self, gc: Handle<U>) -> Ref<'a, U> {
    unsafe {
      let block = &self.storage[gc.index.0];

      assert!(
        block.header.generation() == gc.generation,
        "Use after free."
      );
      Ref::new(
        &block.header,
        NonNull::from_ref(block.data.assume_init_ref()),
      )
    }
  }

  pub fn borrow_mut<'a>(&'a self, gc: Handle<U>) -> RefMut<'a, U> {
    unsafe {
      let block = &self.storage[gc.index.0];
      assert!(gc.generation == block.header.generation(), "Use after free",);
      RefMut::new(
        &block.header,
        NonNull::from_ref(block.data.assume_init_ref()),
      )
    }
  }

  pub fn collect<Root: Trace<U>>(&mut self, root: &Root)
  where
    U: Trace<U> + Debug,
  {
    self.mark(root);
    self.sweep();

    self.cur_generation += 1;
    self.next_gc = self.config.gc_growth_factor * self.bytes_allocated();
  }

  fn mark<Root: Trace<U>>(&self, root: &Root)
  where
    U: Trace<U>,
  {
    let mut tracer = Tracer::new(self, self.cur_generation);
    root.trace(self, &mut tracer);

    while let Some(next) = tracer.pop() {
      unsafe {
        self.borrow_unchecked(next).deref().trace(self, &mut tracer);
      }
    }
  }
  fn sweep(&mut self)
  where
    U: Debug,
  {
    // Ideally we would just use self.block_set, but we want to borrow from
    // self while mutating these...
    let mut next_live_blocks = std::mem::take(&mut self.live_blocks);
    let mut next_free_blocks = std::mem::take(&mut self.free_blocks);

    let removed = next_live_blocks.extract_if(|i| !self.index_header(*i).is_marked());

    for freed_block in removed {
      unsafe {
        debug!(
          "Freeing index {:?}. {:?}",
          freed_block,
          self.storage[freed_block.0].data.assume_init_ref()
        );
      }

      next_free_blocks.push(freed_block);
    }

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
    use itertools::Itertools;
    self.live_blocks.iter().contains(&index)
  }
}

impl<U: Trace<U> + Debug> Trace<U> for Handle<U> {
  fn trace(&self, heap: &Heap<U>, t: &mut Tracer<U>) {
    t.mark(*self);
    unsafe {
      heap.borrow_unchecked(*self).deref().trace(heap, t);
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[derive(Debug)]
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
    let mut heap = Heap::new(HeapConfig {
      initial_size: 4,
      gc_growth_factor: 4,
    });

    let unreachable = heap.alloc(Obj::new()).unwrap();
    let _ = heap.alloc(Obj::with_children(vec![unreachable])).unwrap();

    let reachable = heap.alloc(Obj::new()).unwrap();

    heap.collect(&reachable);

    for i in (0..heap.capacity()).map(Index) {
      if i == reachable.index {
        assert!(heap.is_live(i));
      } else {
        assert!(!heap.is_live(i));
      }
    }
  }

  #[test]
  fn stress_test() {
    let mut heap = Heap::new(HeapConfig {
      initial_size: 4,
      gc_growth_factor: 4,
    });

    let root = heap.alloc(Obj::new()).expect("Root allocation failed");

    for _ in 0..10000 {
      match heap.alloc(Obj::new()) {
        Err(AllocFailure::NeedGc(obj)) => {
          heap.collect(&root);
          heap.alloc(obj).expect("Ran out of space");
        }
        _ => {}
      }
    }

    heap.collect(&root);

    assert!(heap.bytes_allocated() == size_of::<Obj>(),);
  }
}
