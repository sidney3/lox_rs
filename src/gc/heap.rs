use log::info;
use std::fmt;
use std::ptr::NonNull;

type LoxString = String;

#[derive(Debug)]
pub enum ObjData {
  String(LoxString),
}

// the thing that actually gets stored in the GC.
struct GcBlock {
  data: ObjData,
}

#[derive(Clone, Copy, Debug)]
pub struct RootedObj {
  value: NonNull<GcBlock>,
}

#[derive(Clone, Copy, Debug)]
pub struct GcObj {
  value: NonNull<GcBlock>,
}

impl GcBlock {
  pub fn new(data: ObjData) -> Self {
    Self { data }
  }
}

impl GcObj {
  fn new(value: NonNull<GcBlock>) -> Self {
    Self { value }
  }

  pub fn root(self) -> RootedObj {
    RootedObj::new(self.value)
  }
}

impl RootedObj {
  fn new(value: NonNull<GcBlock>) -> Self {
    Self { value }
  }

  pub fn deref(&self) -> &ObjData {
    unsafe { &self.value.as_ref().data }
  }

  pub fn deref_mut(&mut self) -> &mut ObjData {
    unsafe { &mut self.value.as_mut().data }
  }
}

pub struct Heap {
  // TODO: write a real garbage collector and have
  // _real_ object storage.
  storage: Vec<Box<GcBlock>>,
}

impl Heap {
  pub fn new() -> Self {
    Self {
      storage: Vec::new(),
    }
  }

  pub fn alloc(&mut self, obj_data: ObjData) -> GcObj {
    let mut b = Box::new(GcBlock::new(obj_data));
    let obj = GcObj::new(NonNull::new(&raw mut *b).unwrap());
    self.storage.push(b);

    obj
  }
}

impl fmt::Display for ObjData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ObjData::String(s) => write!(f, "{}", s),
    }
  }
}
