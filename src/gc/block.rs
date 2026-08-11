use super::header::GcHeader;
use std::mem::MaybeUninit;

#[repr(C)]
pub struct GcBlock<U> {
  pub header: GcHeader,
  pub data: MaybeUninit<U>,
}

impl<U> GcBlock<U> {
  pub fn new() -> Self {
    Self {
      header: GcHeader::new(),
      data: MaybeUninit::zeroed(),
    }
  }
}
