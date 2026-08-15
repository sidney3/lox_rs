use super::{Closure, Obj, Runtime, UpValue, Value};
use crate::asm::Instruction;
use crate::gc::Ref;

pub struct CallFrame {
  pub closure: Obj<Closure>,
  pub ip: usize,
  pub base: usize,
  pub constants: Vec<Value>,
}

impl CallFrame {
  pub fn jmp(&mut self, to: usize) {
    self.ip = to;
  }
}
