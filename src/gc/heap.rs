use std::fmt;
use std::marker::PhantomData;
use std::ptr::NonNull;

use super::Ref;
use super::block::GcBlock;
use super::header::GcHeader;

#[derive(Debug)]
pub struct Root<'a, U> {
  value: NonNull<GcBlock<U>>,
  _life: PhantomData<&'a Heap<U>>,
}

#[derive(Debug)]
pub struct Handle<U> {
  value: NonNull<GcBlock<U>>,
}

impl<U> Clone for Handle<U> {
  fn clone(&self) -> Self {
    *self
  }
}
impl<U> Copy for Handle<U> {}

impl<U> Handle<U> {
  fn new(value: NonNull<GcBlock<U>>) -> Self {
    Self { value }
  }

  unsafe fn header(&self) -> &GcHeader {
    unsafe { &self.value.as_ref().header }
  }
}

impl<'a, U> Root<'a, U> {
  fn new(value: NonNull<GcBlock<U>>) -> Self {
    Self {
      value,
      _life: PhantomData,
    }
  }

  pub fn borrow(&self) -> Ref<'_, U> {
    unsafe {
      let block = &self.value.as_ref();
      Ref::new(&block.header, NonNull::from_ref(&block.data))
    }
  }
}

pub struct Heap<U> {
  // TODO: write a real garbage collector and have
  // _real_ object storage.
  storage: Vec<Box<GcBlock<U>>>,
}

impl<U> Heap<U> {
  pub fn new() -> Self {
    Self {
      storage: Vec::new(),
    }
  }

  pub fn alloc(&mut self, insertee: U) -> Handle<U> {
    let mut block = Box::new(GcBlock::new(insertee));
    let obj = Handle::new(NonNull::new(&raw mut *block).unwrap());
    self.storage.push(block);

    obj
  }

  pub fn root(&mut self, gc: Handle<U>) -> Root<'_, U> {
    Root::new(gc.value)
  }

  pub fn borrow<'a>(&'a self, gc: Handle<U>) -> Ref<'a, U> {
    unsafe {
      let block = &gc.value.as_ref();
      Ref::new(&block.header, NonNull::from_ref(&block.data))
    }
  }
}

impl<'a, U: fmt::Display> fmt::Display for Root<'a, U> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", *self.borrow())
  }
}
