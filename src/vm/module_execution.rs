use std::fmt;

use log::debug;
use nonempty::{NonEmpty, nonempty};

use super::call_frame::CallFrame;
use super::obj::ObjData;
use super::runtime_error::RuntimeError;
use super::value::Value;
use super::vm::Vm;
use super::vm::{Handle, Root};
use crate::codegen::{Chunk, Constant, Instruction, InstructionKind};
use crate::gc::{Ref, RefMut};

pub struct ModuleExecution<'a, 'b> {
  vm: &'a mut Vm,
  call_stack: NonEmpty<CallFrame<'b>>,
  constants: Vec<Value>,
}

impl<'a, 'b> ModuleExecution<'a, 'b> {
  pub fn new(vm: &'a mut Vm, module: &'b Chunk) -> Self {
    let constants: Vec<_> = module.constants.iter().map(|c| vm.load_const(c)).collect();
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

  fn frame_mut(&mut self) -> &mut CallFrame<'b> {
    self.call_stack.last_mut()
  }

  fn pop_value_always(&mut self) -> Value {
    self.vm.stack.pop().unwrap()
  }

  fn push_value(&mut self, val: Value) {
    self.vm.stack.push(val);
  }

  fn execute_binary<F: FnOnce(Value, Value, &mut Vm) -> Result<Value, RuntimeError>>(
    &mut self,
    f: F,
  ) -> Result<(), RuntimeError> {
    let rhs = self.pop_value_always();
    let lhs = self.pop_value_always();

    let result = f(lhs, rhs, self.vm)?;

    self.push_value(result);

    Ok(())
  }

  fn execute_unary<F: FnOnce(Value, &mut Vm) -> Result<Value, RuntimeError>>(
    &mut self,
    f: F,
  ) -> Result<(), RuntimeError> {
    let operand = self.pop_value_always();

    let result = f(operand, self.vm)?;

    self.push_value(result);
    Ok(())
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
        InstructionKind::Add => self.execute_binary(Value::add)?,
        InstructionKind::Divide => self.execute_binary(Value::divide)?,
        InstructionKind::Mult => self.execute_binary(Value::mult)?,
        InstructionKind::Sub => self.execute_binary(Value::sub)?,
        InstructionKind::Geq => self.execute_binary(Value::geq)?,
        InstructionKind::Leq => self.execute_binary(Value::leq)?,
        InstructionKind::Equals => self.execute_binary(Value::equals)?,
        InstructionKind::Neq => self.execute_binary(Value::neq)?,
        InstructionKind::Greater => self.execute_binary(Value::greater)?,
        InstructionKind::Less => self.execute_binary(Value::less)?,
        InstructionKind::UnaryMinus => self.execute_unary(Value::unary_minus)?,
        InstructionKind::Not => self.execute_unary(Value::not)?,
        InstructionKind::Constant => {
          let val = self.constants[next_instruction.operand as usize];
          debug!("CONST {:?}", val);
          self.push_value(val);
        }
        InstructionKind::Pop => {
          let val = self.vm.stack.pop().unwrap();

          debug!("POP {:?}", val);
        }
        InstructionKind::Print => {
          let val = self.vm.stack.pop().unwrap();
          let repr = match val {
            Value::Num(x) => x.to_string(),
            Value::Obj(handle) => {
              let obj = self.vm.heap.root(handle);
              format!("{}", *obj.borrow())
            }
            Value::Bool(b) => if b { "True" } else { "False" }.to_string(),
          };
          println!("{repr}");
          debug!("PRINT {:?}", val);
        }
        InstructionKind::Assert => {
          let operand = match self.pop_value_always() {
            Value::Bool(b) => b,
            _ => return Err(RuntimeError::new("Assert expects bool operand")),
          };

          if !operand {
            return Err(RuntimeError::new("Assert failed"));
          }
        }
      }
    }
  }
}
