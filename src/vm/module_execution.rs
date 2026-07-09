use log::debug;
use nonempty::{NonEmpty, nonempty};

use super::call_frame::CallFrame;
use super::runtime_error::RuntimeError;
use super::value::Value;
use super::vm::Vm;
use crate::codegen::{Chunk, Constant, Instruction, InstructionKind};

pub struct ModuleExecution<'a, 'b> {
  vm: &'a mut Vm,
  call_stack: NonEmpty<CallFrame<'b>>,
  constants: Vec<Value>,
}

enum StepSignal {
  Stop,
  Continue,
  Abort,
}

impl<'a, 'b> ModuleExecution<'a, 'b> {
  pub fn new(vm: &'a mut Vm, module: &'b Chunk) -> Self {
    let mut constants = Vec::new();

    for x in &module.constants {
      let value = match x {
        Constant::Float(f) => Value::Num(*f),
      };
      constants.push(value);
    }
    Self {
      vm,
      call_stack: nonempty![CallFrame {
        chunk: module,
        ip: 0,
        base: 0,
      }],
      constants,
    }
  }

  pub fn frame_mut(&mut self) -> &mut CallFrame<'b> {
    self.call_stack.last_mut()
  }

  pub fn pop_value(&mut self) -> Option<Value> {
    self.vm.stack.pop()
  }

  pub fn pop_value_always(&mut self) -> Value {
    self.vm.stack.pop().unwrap()
  }

  pub fn push_value(&mut self, val: Value) {
    self.vm.stack.push(val);
  }

  pub fn execute(mut self) -> Result<(), RuntimeError> {
    loop {
      let next_instruction: Instruction = if let Some(inst) = self.frame_mut().pop_instruction() {
        inst
      } else {
        panic!("Walked off the end of frame!")
      };

      match next_instruction.kind {
        InstructionKind::Return => {
          debug!("RETURN");
          return Ok(());
        }
        InstructionKind::Add => {
          let lhs = self.pop_value_always();
          let rhs = self.pop_value_always();
          let result = match (&lhs, &rhs) {
            (Value::Num(a), Value::Num(b)) => Value::Num(a + b),
          };

          debug!("ADD {:?} {:?}", lhs, rhs);
          self.push_value(result);
        }
        InstructionKind::Constant => {
          let val = self.constants[next_instruction.operand as usize].clone();
          debug!("CONST {:?}", val);
          self.push_value(val);
        }
        InstructionKind::Pop => {
          let val = self.vm.stack.pop().unwrap();

          debug!("POP {:?}", val);
        }
        InstructionKind::Print => {
          let val = self.vm.stack.pop().unwrap();
          println!("{:?}", val);
          debug!("PRINT {:?}", val);
        }

        _ => todo!("Unsupported instruction"),
      }
    }
  }
}
