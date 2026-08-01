use std::ops::Deref;

use lasso::Key;
use log::debug;
use nonempty::{NonEmpty, nonempty};

use super::call_frame::CallFrame;
use crate::asm::{Instruction, InstructionKind};
use crate::runtime::{Closure, Function, Handle, Obj, ObjData, Runtime, RuntimeError, Value};

pub struct Executor<'vm> {
  vm: &'vm mut Runtime,
  call_stack: NonEmpty<CallFrame>,
}

impl<'vm> Executor<'vm> {
  pub fn new(vm: &'vm mut Runtime, main: Obj<Function>) -> Self {
    let main_closure = vm.alloc_typed(Closure { func: main });
    let main_frame = CallFrame::new(vm, main_closure, 0);

    Self {
      vm,
      call_stack: nonempty![main_frame],
    }
  }

  pub fn run(self) -> Result<(), RuntimeError> {
    self.execute()
  }

  fn pop(&mut self) -> Value {
    self
      .vm
      .value_stack
      .pop()
      .expect("empty stack. Programming error")
  }

  fn peek(&mut self) -> &Value {
    self
      .vm
      .value_stack
      .last()
      .expect("empty stack. Programming error")
  }

  fn load_stack(&self, offset: usize) -> Value {
    self
      .call_stack
      .last()
      .load_val(offset, &self.vm.value_stack)
      .expect("Stack offset out of range")
  }
  fn set_stack(&mut self, offset: usize, set_to: Value) {
    self
      .call_stack
      .last()
      .set_val(offset, &mut self.vm.value_stack, set_to)
      .expect("Stack offset out of range");
  }

  fn push_value(&mut self, val: Value) {
    self.vm.value_stack.push(val);
  }

  fn execute_binary<F: FnOnce(Value, Value, &mut Runtime) -> Result<Value, RuntimeError>>(
    &mut self,
    f: F,
  ) -> Result<(), RuntimeError> {
    let rhs = self.pop();
    let lhs = self.pop();

    let result = f(lhs, rhs, self.vm)?;

    self.push_value(result);

    Ok(())
  }

  fn execute_unary<F: FnOnce(Value, &mut Runtime) -> Result<Value, RuntimeError>>(
    &mut self,
    f: F,
  ) -> Result<(), RuntimeError> {
    let operand = self.pop();

    let result = f(operand, self.vm)?;

    self.push_value(result);
    Ok(())
  }

  pub fn execute(mut self) -> Result<(), RuntimeError> {
    loop {
      let next_instruction: Instruction =
        if let Some(inst) = self.call_stack.last_mut().pop_instruction(&self.vm) {
          inst
        } else {
          panic!("Walked off the end of frame!")
        };
      debug!(
        "About to execute: {:?}. Ip = {}, Bp = {}, Sp = {}",
        next_instruction,
        self.call_stack.last().ip,
        self.call_stack.last().base,
        self.vm.value_stack.len()
      );

      match next_instruction.kind {
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
          debug!("{:?}", self.call_stack.last().constants);
          let val = self.call_stack.last().constants[next_instruction.operand as usize];
          self.push_value(val);
        }
        InstructionKind::Pop => {
          self.pop();
        }
        InstructionKind::Print => {
          let val = self.pop();
          println!("{}", val.repr(self.vm));
        }
        InstructionKind::JumpIfFalse => {
          if !bool::try_from(&self.pop())? {
            self
              .call_stack
              .last_mut()
              .jmp(&self.vm, next_instruction.operand as usize)
          }
        }
        InstructionKind::JumpIfFalsePreserving => {
          if !bool::try_from(self.peek())? {
            self
              .call_stack
              .last_mut()
              .jmp(&self.vm, next_instruction.operand as usize)
          }
        }
        InstructionKind::JumpIfTruePreserving => {
          if bool::try_from(self.peek())? {
            self
              .call_stack
              .last_mut()
              .jmp(&self.vm, next_instruction.operand as usize)
          }
        }
        InstructionKind::Jmp => {
          self
            .call_stack
            .last_mut()
            .jmp(&self.vm, next_instruction.operand as usize);
        }
        InstructionKind::Assert => {
          let operand = match self.pop() {
            Value::Bool(b) => b,
            _ => return Err(RuntimeError::new("Assert expects bool operand")),
          };

          if !operand {
            return Err(RuntimeError::new("Assert failed"));
          }
        }
        InstructionKind::AddGlobal => {
          let assign = self.pop();
          let global_idx =
            lasso::Spur::try_from_usize(next_instruction.operand as usize).expect("Bad spur");

          self.vm.globals.insert(global_idx, assign);
        }
        InstructionKind::LoadGlobal => {
          let global_idx =
            lasso::Spur::try_from_usize(next_instruction.operand as usize).expect("Bad spur");

          let global = self.vm.globals.get(&global_idx).ok_or_else(|| {
            RuntimeError::new(
              format!(
                "Unrecognized ident: {}",
                self.vm.symbols.resolve(&global_idx)
              )
              .as_str(),
            )
          })?;

          self.push_value(*global);
        }
        InstructionKind::LoadLocal => {
          let stack_offset = next_instruction.operand as usize;
          let val = self.load_stack(stack_offset);
          self.push_value(val);
        }
        InstructionKind::SetLocal => {
          let assign: Value = self.pop();
          let stack_offset = next_instruction.operand as usize;
          self.set_stack(stack_offset, assign);
        }
        InstructionKind::SetGlobal => {
          let assign: Value = self.pop();
          let global_idx =
            lasso::Spur::try_from_usize(next_instruction.operand as usize).expect("Bad spur");

          let global: &mut Value = self.vm.globals.get_mut(&global_idx).ok_or_else(|| {
            RuntimeError::new(
              format!(
                "Unrecognized ident: {}",
                self.vm.symbols.resolve(&global_idx)
              )
              .as_str(),
            )
          })?;
          *global = assign;
        }

        InstructionKind::Callq => {
          let nargs = next_instruction.operand as usize;
          let f_idx = self.vm.stack_top() - nargs - 1;
          let handle = match self.vm.value_stack[f_idx] {
            Value::Obj(obj) => obj,
            _ => return Err(RuntimeError::new("Expected function")),
          };

          let func = match Obj::<Function>::downcast(handle, self.vm) {
            Some(func) => func,
            None => return Err(RuntimeError::new("Call can only be called on a function")),
          };

          let func_closure = self.vm.alloc_typed(Closure { func });

          self
            .call_stack
            .push(CallFrame::new(self.vm, func_closure, f_idx));
        }
        InstructionKind::Return => {
          let returned_frame = match self.call_stack.pop() {
            Some(called) => called,
            None => {
              assert!(self.vm.value_stack.is_empty());
              assert!(self.call_stack.last().base == 0);
              return Ok(());
            }
          };
          let ret_val = self.pop();

          let leftover_stack_vals = returned_frame
            .closure
            .borrow(self.vm)
            .borrow_func(self.vm)
            .arity
            + 1;

          assert!(
            self.vm.value_stack.len() == returned_frame.base + leftover_stack_vals,
            "We got calling conventions wrong. Stack sized {} when expecting bp={} + (f,args...)={}.",
            self.vm.value_stack.len(),
            returned_frame.base,
            leftover_stack_vals,
          );

          while self.vm.value_stack.len() > returned_frame.base {
            self.vm.value_stack.pop();
          }

          self.push_value(ret_val);
        }
      }
    }
  }
}
