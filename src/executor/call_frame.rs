use crate::asm::Instruction;
use crate::gc::Ref;
use crate::runtime::{Closure, Function, Handle, Obj, ObjData, Runtime, UpValue, Value};

pub struct CallFrame {
  pub closure: Obj<Closure>,
  pub ip: usize,
  pub base: usize,

  pub constants: Vec<Value>,
}

impl CallFrame {
  pub fn new(vm: &mut Runtime, closure: Obj<Closure>, base: usize) -> Self {
    let constants: Vec<Value> = closure.borrow(vm).func.borrow(vm).chunk.constants.clone();

    Self {
      closure,
      ip: 0,
      base,
      constants,
    }
  }

  pub fn upvalues<'a>(&self, vm: &'a Runtime) -> Ref<'a, Vec<Obj<UpValue>>> {
    Ref::map(self.closure.borrow(vm), |closure| &closure.upvalues)
  }

  pub fn func<'a>(&self, vm: &'a Runtime) -> Ref<'a, Function> {
    self.closure.borrow(vm).func.borrow(vm)
  }

  pub fn pop_instruction(&mut self, vm: &Runtime) -> Option<Instruction> {
    match self.func(vm).chunk.instructions.get(self.ip) {
      Some(&instruction) => {
        self.ip += 1;
        Some(instruction)
      }
      None => None,
    }
  }

  pub fn jmp(&mut self, vm: &Runtime, to: usize) {
    assert!(to < self.func(vm).chunk.instructions.len());

    self.ip = to;
  }

  pub fn load_val(&self, offset: usize, value_stack: &Vec<Value>) -> Option<Value> {
    value_stack.get(self.base + offset).cloned()
  }
  pub fn set_val(&self, offset: usize, value_stack: &mut Vec<Value>, set_to: Value) -> Option<()> {
    let idx = self.base + offset;
    if idx < value_stack.len() {
      value_stack[idx] = set_to;
      Some(())
    } else {
      None
    }
  }
}
