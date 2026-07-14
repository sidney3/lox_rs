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
use std::fmt;

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

  fn obj(&self, h: Handle) -> Ref<'_, ObjData> {
    self.vm.heap.borrow(h)
  }

  fn execute_binary<F: FnOnce(&mut Self, Value, Value) -> Result<Value, RuntimeError>>(
    &mut self,
    f: F,
  ) -> Result<(), RuntimeError> {
    let rhs = self.pop_value_always();
    let lhs = self.pop_value_always();

    let result = f(self, lhs, rhs)?;

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
        InstructionKind::Add => self.execute_binary(|s, lhs, rhs| {
          debug!("ADD {:?} {:?}", lhs, rhs);
          match (&lhs, &rhs) {
            (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
            (&Value::Obj(a), &Value::Num(b)) => {
              let obj = s.obj(a).add_left(b)?;
              Ok(s.vm.alloc(obj))
            }
            (&Value::Num(a), &Value::Obj(b)) => {
              let obj = s.obj(b).add_right(a)?;
              Ok(s.vm.alloc(obj))
            }
            _ => Err(RuntimeError {
              msg: format!("Bad binary expression between: {:?}, {:?}", lhs, rhs,),
            }),
          }
        })?,
        InstructionKind::Divide => self.execute_binary(|_, lhs, rhs| match (&lhs, &rhs) {
          (Value::Num(a), &Value::Num(b)) => {
            if b != 0f64 {
              Ok(Value::Num(a / b))
            } else {
              Err(RuntimeError {
                msg: "Divide by zero".to_string(),
              })
            }
          }
          _ => Err(RuntimeError {
            msg: format!("Bad divide between: {:?}, {:?}", lhs, rhs),
          }),
        })?,
        InstructionKind::Mult => self.execute_binary(|_, lhs, rhs| match (&lhs, &rhs) {
          (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a * b)),
          _ => Err(RuntimeError {
            msg: format!("Bad mult between: {:?}, {:?}", lhs, rhs),
          }),
        })?,
        InstructionKind::Sub => self.execute_binary(|_, lhs, rhs| match (&lhs, &rhs) {
          (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a - b)),
          _ => Err(RuntimeError {
            msg: format!("Bad sub between: {:?}, {:?}", lhs, rhs),
          }),
        })?,
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
          let repr = match val {
            Value::Num(x) => x.to_string(),
            Value::Obj(handle) => {
              let obj = self.vm.heap.root(handle);
              format!("{}", *obj.borrow())
            }
          };
          println!("{repr}");
          debug!("PRINT {:?}", val);
        }
        _ => todo!("unhandled instruction: {}", next_instruction.kind),
      }
    }
  }
}
