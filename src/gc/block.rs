use super::header::GcHeader;
use std::mem::MaybeUninit;

#[repr(C)]
pub struct GcBlock<U> {
  pub header: GcHeader,
  pub data: MaybeUninit<U>,
}

impl<U> GcBlock<U> {
  pub fn new(valid_until: usize) -> Self {
    Self {
      header: GcHeader::new(valid_until),
      data: MaybeUninit::zeroed(),
    }
  }
}
