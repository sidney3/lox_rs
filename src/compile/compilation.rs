use crate::runtime::{Function, Handle, Obj, Value};

pub struct Compilation {
  pub main: Obj<Function>,
}
