use super::block::GcBlock;
use super::header::GcHeader;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub struct Ref<'a, U> {
  header: &'a GcHeader,
  value: NonNull<U>,
  _life: PhantomData<&'a U>,
}

impl<'a, U> Ref<'a, U> {
  pub fn map<T, F: FnOnce(&U) -> &T>(r: Self, f: F) -> Ref<'a, T> {
    unsafe {
      let next_ref = f(r.value.as_ref());
      Ref::new(r.header, NonNull::from_ref(next_ref))
    }
  }

  pub unsafe fn new(header: &'a GcHeader, value: NonNull<U>) -> Self {
    header.start_borrow();

    Self {
      header,
      value,
      _life: PhantomData,
    }
  }
}

impl<'a, U> Deref for Ref<'a, U> {
  type Target = U;
  fn deref(&self) -> &Self::Target {
    unsafe { &self.value.as_ref() }
  }
}

impl<'a, U> Drop for Ref<'a, U> {
  fn drop(&mut self) {
    self.header.end_borrow();
  }
}
