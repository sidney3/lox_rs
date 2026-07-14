use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use log::info;

#[derive(Clone, Copy)]
enum BorrowState {
  Unborrowed,
  Shared(u32),
  BorrowedMut,
}

struct GcHeader {
  borrow_state: Cell<BorrowState>,
}

impl GcHeader {
  pub fn new() -> Self {
    Self {
      borrow_state: Cell::new(BorrowState::Unborrowed),
    }
  }

  // runtime borrow checker
  pub fn start_borrow(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::Unborrowed => BorrowState::Shared(1),
      BorrowState::Shared(k) => BorrowState::Shared(k + 1),
      BorrowState::BorrowedMut => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }

  pub fn end_borrow(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::Shared(k) if k == 1 => BorrowState::Unborrowed,
      BorrowState::Shared(k) if k > 1 => BorrowState::Shared(k - 1),
      _ => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }

  pub fn start_borrow_mut(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::Unborrowed => BorrowState::BorrowedMut,
      _ => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }

  pub fn end_borrow_mut(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::BorrowedMut => BorrowState::Unborrowed,
      _ => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }
}

pub struct Ref<'a, U> {
  value: NonNull<GcBlock<U>>,
  _life: PhantomData<&'a U>,
}

pub struct RefMut<'a, U> {
  value: NonNull<GcBlock<U>>,
  _life: PhantomData<&'a mut U>,
}

impl<'a, U> Deref for Ref<'a, U> {
  type Target = U;
  fn deref(&self) -> &Self::Target {
    unsafe { &self.value.as_ref().data }
  }
}

impl<'a, U> Deref for RefMut<'a, U> {
  type Target = U;
  fn deref(&self) -> &Self::Target {
    unsafe { &self.value.as_ref().data }
  }
}

impl<'a, U> DerefMut for RefMut<'a, U> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut self.value.as_mut().data }
  }
}

impl<'a, U> Drop for Ref<'a, U> {
  fn drop(&mut self) {
    unsafe {
      self.value.as_ref().header.end_borrow();
    }
  }
}

impl<'a, U> Drop for RefMut<'a, U> {
  fn drop(&mut self) {
    unsafe {
      self.value.as_ref().header.end_borrow();
    }
  }
}

struct GcBlock<U> {
  header: GcHeader,
  data: U,
}

#[derive(Debug)]
pub struct Root<U> {
  value: NonNull<GcBlock<U>>,
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

impl<U> GcBlock<U> {
  pub fn new(data: U) -> Self {
    Self {
      header: GcHeader::new(),
      data,
    }
  }
}

impl<U> Handle<U> {
  fn new(value: NonNull<GcBlock<U>>) -> Self {
    Self { value }
  }

  unsafe fn header(&self) -> &GcHeader {
    unsafe { &self.value.as_ref().header }
  }
}

impl<U> Root<U> {
  fn new(value: NonNull<GcBlock<U>>) -> Self {
    Self { value }
  }

  unsafe fn header(&self) -> &GcHeader {
    unsafe { &self.value.as_ref().header }
  }

  pub fn borrow(&self) -> Ref<'_, U> {
    unsafe {
      self.header().start_borrow();
    }
    Ref {
      value: self.value,
      _life: PhantomData,
    }
  }

  pub fn borrow_mut(&mut self) -> RefMut<'_, U> {
    unsafe {
      self.header().start_borrow_mut();
    }
    RefMut {
      value: self.value,
      _life: PhantomData,
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

  pub fn root(&mut self, gc: Handle<U>) -> Root<U> {
    Root::new(gc.value)
  }

  pub fn borrow(&self, gc: Handle<U>) -> Ref<'_, U> {
    unsafe {
      gc.header().start_borrow();
    }

    Ref {
      value: gc.value,
      _life: PhantomData,
    }
  }
  pub fn borrow_mut(&self, gc: Handle<U>) -> RefMut<'_, U> {
    unsafe {
      gc.header().start_borrow_mut();
    }

    RefMut {
      value: gc.value,
      _life: PhantomData,
    }
  }
}

impl<U: fmt::Display> fmt::Display for Root<U> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", *self.borrow())
  }
}
