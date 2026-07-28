use super::header::GcHeader;

#[repr(C)]
pub struct GcBlock<U> {
  pub header: GcHeader,
  pub data: U,
}

impl<U> GcBlock<U> {
  pub fn new(data: U) -> Self {
    Self {
      header: GcHeader::new(),
      data,
    }
  }
}
