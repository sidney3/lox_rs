use log::debug;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use super::header::GcHeader;

pub struct Ref<'a, U> {
  header: &'a GcHeader,
  value: NonNull<U>,
  _life: PhantomData<&'a U>,
}

impl<'a, U> Ref<'a, U> {
  pub fn map<T, F: FnOnce(&U) -> &T>(r: Self, f: F) -> Ref<'a, T> {
    unsafe {
      let this = ManuallyDrop::new(r);
      let next_ref = f(this.value.as_ref());
      Ref::from_other(this.header, NonNull::from_ref(next_ref))
    }
  }

  unsafe fn from_other(header: &'a GcHeader, value: NonNull<U>) -> Self {
    Self {
      header,
      value,
      _life: PhantomData,
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
    unsafe { self.value.as_ref() }
  }
}

impl<'a, U> Drop for Ref<'a, U> {
  fn drop(&mut self) {
    self.header.end_borrow();
  }
}

pub struct RefMut<'a, U> {
  header: &'a GcHeader,
  value: NonNull<U>,
  _life: PhantomData<&'a U>,
}

impl<'a, U> RefMut<'a, U> {
  pub fn map<T, F: FnOnce(&U) -> &T>(r: Self, f: F) -> RefMut<'a, T> {
    unsafe {
      let this = ManuallyDrop::new(r);
      let next_ref = f(this.value.as_ref());
      RefMut::from_other(this.header, NonNull::from_ref(next_ref))
    }
  }

  fn from_other(header: &'a GcHeader, value: NonNull<U>) -> Self {
    Self {
      header,
      value,
      _life: PhantomData,
    }
  }
  pub unsafe fn new(header: &'a GcHeader, value: NonNull<U>) -> Self {
    header.start_borrow_mut();

    Self {
      header,
      value,
      _life: PhantomData,
    }
  }
}

impl<'a, U> DerefMut for RefMut<'a, U> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.value.as_mut() }
  }
}

impl<'a, U> Deref for RefMut<'a, U> {
  type Target = U;
  fn deref(&self) -> &Self::Target {
    unsafe { self.value.as_ref() }
  }
}

impl<'a, U> Drop for RefMut<'a, U> {
  fn drop(&mut self) {
    self.header.end_borrow_mut();
  }
}
