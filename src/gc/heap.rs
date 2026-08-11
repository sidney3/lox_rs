use super::header::GcHeader;
use std::marker::PhantomData;
use std::ptr::NonNull;

use super::block::GcBlock;
use super::{Ref, RefMut};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
  free_blocks: Vec<Index>,
}

impl<U> Heap<U> {
  pub fn new(initial_size: usize) -> Self {
    let mut heap = Self {
      storage: Vec::new(),
      free_blocks: Vec::new(),
    };
    heap.grow(initial_size);
    heap
  }

  fn grow(&mut self, by: usize) {
    let old_len = self.storage.len();
    let new_indices = (0..by).map(|i| Index(old_len + i));

    self
      .storage
      .resize_with(self.storage.len() + by, GcBlock::<U>::new);
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

    Handle::<U>::new(block)
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

  pub(super) fn header_mut(&mut self, index: Index) -> &mut GcHeader {
    &mut self.storage[index.0].header
  }
}
