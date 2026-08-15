use super::{Closure, Obj, Runtime, UpValue, Value};
use crate::asm::Instruction;
use crate::gc::{Heap, Ref, Trace, Tracer};
use crate::runtime::ObjData;

pub struct CallFrame {
  pub closure: Obj<Closure>,
  pub ip: usize,
  pub base: usize,
  pub constants: Vec<Value>,
}

impl Trace<ObjData> for CallFrame {
  fn trace(&self, heap: &Heap<ObjData>, tracer: &mut Tracer<ObjData>) {
    tracer.mark(self.closure.as_handle());

    for constant in &self.constants {
      constant.trace(heap, tracer);
    }
  }
}

impl CallFrame {
  pub fn jmp(&mut self, to: usize) {
    self.ip = to;
  }
}
