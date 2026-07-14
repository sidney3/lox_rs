use std::fmt;

use super::vm::Handle;

#[derive(Debug, Clone)]
pub enum Value {
  Num(f64),
  Obj(Handle),
}
