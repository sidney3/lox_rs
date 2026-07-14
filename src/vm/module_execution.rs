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
            (&Value::Obj(a), &Value::Num(b)) => {
              let obj = self.obj(a).add_left(b)?;
              self.vm.alloc(obj)
            }
            (&Value::Num(a), &Value::Obj(b)) => {
              let obj = self.obj(b).add_right(a)?;
              self.vm.alloc(obj)
            }
            _ => {
              return Err(RuntimeError {
                msg: format!("Bad binary expression between: {:?}, {:?}", lhs, rhs,),
              });
            }
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

        _ => todo!("Unsupported instruction"),
      }
    }
  }
}
