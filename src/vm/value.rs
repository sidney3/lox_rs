use std::fmt;

use super::vm::Handle;

pub type Num = f64;

#[derive(Debug, Clone)]
pub enum Value {
  Num(Num),
  Obj(Handle),
}
