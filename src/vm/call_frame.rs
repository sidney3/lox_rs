use super::obj::Function;
use super::obj::ObjData;
use super::value::Value;
use super::vm::{Handle, Vm};
use crate::asm::Constant;
use crate::asm::{Chunk, Instruction};
use crate::gc::Ref;
use std::ops::Deref;

pub struct CallFrame {
  pub func: Handle,
  pub ip: usize,
  pub base: usize,

  pub constants: Vec<Value>,
}

impl CallFrame {
  pub fn new(vm: &mut Vm, func: Handle, base: usize) -> Self {
    let extern_constants: Vec<Constant> = match vm.heap.borrow(func).deref() {
      ObjData::Func(f) => f.chunk.constants.iter().cloned().collect(),
      _ => panic!("CallFrame called on not a function"),
    };

    let constants = extern_constants
      .into_iter()
      .map(|c| vm.load_const(c))
      .collect();

    Self {
      func,
      ip: 0,
      base,
      constants,
    }
  }
  pub fn func<'vm>(&self, vm: &'vm Vm) -> Ref<'vm, Function> {
    Ref::map(vm.heap.borrow(self.func), |obj_data| match obj_data {
      ObjData::Func(f) => f,
      _ => panic!("CallFrame.func is always a function"),
    })
  }

  pub fn pop_instruction(&mut self, vm: &Vm) -> Option<Instruction> {
    match self.func(vm).chunk.instructions.get(self.ip) {
      Some(&instruction) => {
        self.ip += 1;
        Some(instruction)
      }
      None => None,
    }
  }

  pub fn jmp(&mut self, vm: &Vm, to: usize) {
    assert!(to < self.func(vm).chunk.instructions.len());

    self.ip = to;
  }

  pub fn load_val<'vm>(&self, offset: usize, value_stack: &Vec<Value>) -> Option<Value> {
    value_stack.get(self.base + offset).cloned()
  }
  pub fn set_val<'vm>(
    &self,
    offset: usize,
    value_stack: &mut Vec<Value>,
    set_to: Value,
  ) -> Option<()> {
    let idx = self.base + offset;
    if idx < value_stack.len() {
      value_stack[idx] = set_to;
      Some(())
    } else {
      None
    }
  }
}
