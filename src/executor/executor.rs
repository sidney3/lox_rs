use std::ops::Deref;

use lasso::Key;
use log::{debug, info};
use nonempty::{NonEmpty, nonempty};

use super::call_frame::CallFrame;
use crate::asm::{Instruction, InstructionKind};
use crate::runtime::{
  Closure, Function, Handle, Obj, ObjData, Runtime, RuntimeError, UpValue, UpValueDescriptor, Value,
};

pub struct Executor<'vm> {
  vm: &'vm mut Runtime,
  call_stack: NonEmpty<CallFrame>,
}

impl<'vm> Executor<'vm> {
  pub fn new(vm: &'vm mut Runtime, main: Obj<Function>) -> Self {
    assert!(
      main.borrow(vm).upvalues.is_empty(),
      "main should not capture any closure references"
    );
    let main_closure = vm.alloc_typed(Closure {
      func: main,
      upvalues: Vec::new(),
    });
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

  fn frame(&self) -> &CallFrame {
    self.call_stack.last()
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

        InstructionKind::MakeClosure => {
          let func_handle = self.call_stack.last().constants[next_instruction.operand as usize];

          let func = match func_handle {
            Value::Obj(handle) => {
              Obj::<Function>::downcast(handle, self.vm).expect("MakeClosure expects a function")
            }
            _ => panic!("MakeClosure expects a function"),
          };

          let mut upvalues: Vec<Obj<UpValue>> = Vec::new();

          let upvalue_descs = func.borrow(self.vm).upvalues.clone();

          for (_, upvalue_desc) in upvalue_descs {
            match upvalue_desc {
              UpValueDescriptor::Local { parent_stack_pos } => {
                let absolute_stack_pos = self.frame().base + parent_stack_pos;
                upvalues.push(self.vm.alloc_typed(UpValue::Open { absolute_stack_pos }));
              }
              UpValueDescriptor::Recursive { parent_upvalue_pos } => {
                let parent = self.call_stack.last();

                info!("Parent upvalues: {:?}", parent.upvalues(self.vm).deref());

                upvalues.push(parent.upvalues(self.vm)[parent_upvalue_pos]);
              }
            }
          }

          info!(
            "Func: {:?} with upvalues {:?}",
            &func.borrow(self.vm).name,
            upvalues
          );

          let closure = self.vm.alloc(ObjData::Closure(Closure { func, upvalues }));

          self.push_value(Value::Obj(closure));
        }
        InstructionKind::Callq => {
          let nargs = next_instruction.operand as usize;
          let f_idx = self.vm.stack_top() - nargs - 1;
          let handle = match self.vm.value_stack[f_idx] {
            Value::Obj(obj) => obj,
            _ => return Err(RuntimeError::new("Expected function")),
          };

          let func = match Obj::<Closure>::downcast(handle, self.vm) {
            Some(func) => func,
            None => return Err(RuntimeError::new("Call can only be called on a function")),
          };

          self.call_stack.push(CallFrame::new(self.vm, func, f_idx));
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

          let leftover_stack_vals = returned_frame.func(self.vm).arity + 1;

          assert!(
            self.vm.value_stack.len() >= returned_frame.base + leftover_stack_vals,
            "We got calling conventions wrong. Stack sized {} when at least expecting bp={} + (f,args...)={}.",
            self.vm.value_stack.len(),
            returned_frame.base,
            leftover_stack_vals,
          );

          while self.vm.value_stack.len() > returned_frame.base {
            self.vm.value_stack.pop();
          }

          self.push_value(ret_val);
        }
        InstructionKind::LoadUpValue => {
          let stack_pos = {
            let idx = next_instruction.operand as usize;
            let upvalue = &self.frame().upvalues(self.vm)[idx];

            match upvalue.borrow(self.vm).deref() {
              &UpValue::Open { absolute_stack_pos } => absolute_stack_pos,
            }
          };

          // TODO: when we add promotion from stack -> heap, make some
          // enum that models "load strategy" or something
          self.push_value(self.vm.value_stack[stack_pos]);
        }
        InstructionKind::SetUpValue => {
          let set_to = self.pop();

          let stack_pos = {
            match self.frame().upvalues(self.vm)[next_instruction.operand as usize]
              .borrow(self.vm)
              .deref()
            {
              &UpValue::Open { absolute_stack_pos } => absolute_stack_pos,
            }
          };
          self.vm.value_stack[stack_pos] = set_to;
        }
      }
    }
  }
}
