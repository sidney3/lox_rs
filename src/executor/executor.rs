use std::ops::Deref;

use lasso::Key;
use log::debug;
use nonempty::{NonEmpty, nonempty};
use smallvec::SmallVec;

use crate::asm::{Instruction, InstructionKind};
use crate::gc::Ref;
use crate::runtime::{
  CallFrame, Closure, FrameIndex, Function, Obj, ObjData, Root, Runtime, RuntimeError, UpValue,
  UpValueDescriptor, Value,
};
use either::Either;

pub struct Executor<'vm> {
  vm: &'vm mut Runtime,
  call_stack: NonEmpty<FrameIndex>,
  open_upvalues: Vec<Obj<UpValue>>,
}

impl<'vm> Executor<'vm> {
  pub fn new(vm: &'vm mut Runtime, main: Root<Function>) -> Self {
    assert!(
      main.as_obj().borrow(vm).upvalues.is_empty(),
      "main should not capture any closure references"
    );

    let main_closure = vm
      .alloc_typed(Closure {
        func: main.as_obj(),
        upvalues: Vec::new(),
      })
      .as_root(vm);

    // lifetime extended by main closure
    main.free(vm);

    let main_frame = vm.alloc_frame(main_closure.as_obj(), 0);

    // lifetime extended vm frames
    main_closure.free(vm);

    Self {
      vm,
      call_stack: nonempty![main_frame],
      open_upvalues: Vec::new(),
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
    self.vm.frame(*self.call_stack.last())
  }
  fn frame_mut(&mut self) -> &mut CallFrame {
    self.vm.frame_mut(*self.call_stack.last())
  }

  fn load_instruction(&mut self) -> Option<Instruction> {
    let frame = self.frame();
    let out = frame
      .closure
      .borrow(self.vm)
      .func
      .borrow(self.vm)
      .instructions
      .get(frame.ip)
      .cloned();

    self.frame_mut().ip += if out.is_some() { 1 } else { 0 };

    out
  }
  fn load_stack(&self, offset: usize) -> Value {
    let frame = self.frame();

    self.vm.value_stack[frame.base + offset]
  }
  fn load_const(&self, index: usize) -> Value {
    self.frame().constants[index]
  }
  fn set_stack(&mut self, offset: usize, set_to: Value) {
    let idx = offset + self.frame().base;
    self.vm.value_stack[idx] = set_to;
  }

  fn active_fn(&self) -> Ref<'_, Closure> {
    self.frame().closure.borrow(self.vm)
  }

  fn upval(&self, idx: usize) -> Obj<UpValue> {
    self.active_fn().upvalues[idx]
  }

  fn push_value(&mut self, val: Value) {
    self.vm.value_stack.push(val);
  }

  fn execute_binary<F: FnOnce(Value, Value, &Runtime) -> Result<Value, RuntimeError>>(
    &mut self,
    f: F,
  ) -> Result<(), RuntimeError> {
    let rhs = self.pop();
    let lhs = self.pop();

    let result = f(lhs, rhs, self.vm)?;

    self.push_value(result);

    Ok(())
  }

  fn execute_unary<F: FnOnce(Value, &Runtime) -> Result<Value, RuntimeError>>(
    &mut self,
    f: F,
  ) -> Result<(), RuntimeError> {
    let operand = self.pop();

    let result = f(operand, self.vm)?;

    self.push_value(result);
    Ok(())
  }

  // Drain the stack, specifically checking for upvalues
  //
  // until is inclusive. e.g. AFTER drain_stack runs,
  // value_stack.len() == until
  fn drain_stack(&mut self, until: usize) {
    let drained_upvals: SmallVec<[Obj<UpValue>; 4]> = {
      self
        .open_upvalues
        .extract_if(.., |upval| match upval.borrow(self.vm).deref() {
          &UpValue::Open { absolute_stack_pos } => absolute_stack_pos >= until,
          _ => panic!("open_upvalues array should just be open upvalues"),
        })
        .collect()
    };

    for upval in drained_upvals {
      let stack_pos = {
        match upval.borrow(self.vm).deref() {
          &UpValue::Open { absolute_stack_pos } => absolute_stack_pos,
          _ => panic!("unreachable"),
        }
      };

      *upval.borrow_mut(self.vm) = UpValue::Closed(self.vm.value_stack[stack_pos]);
    }

    self.vm.value_stack.drain(until..);
  }

  pub fn execute(mut self) -> Result<(), RuntimeError> {
    loop {
      let next_instruction: Instruction = if let Some(inst) = self.load_instruction() {
        inst
      } else {
        panic!("Walked off the end of frame!")
      };
      debug!(
        "About to execute: {:?}. Ip = {}, Bp = {}, Sp = {}",
        next_instruction,
        self.frame().ip,
        self.frame().base,
        self.vm.value_stack.len()
      );

      match next_instruction.kind {
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
        InstructionKind::Add => {
          let rhs = self.pop();
          let lhs = self.pop();
          let proto_val = lhs.add(rhs, self.vm)?;
          let val = proto_val.into_value(self.vm);
          self.push_value(val);
        }
        InstructionKind::Constant => {
          let val = self.load_const(next_instruction.operand as usize);
          self.push_value(val);
        }
        InstructionKind::Pop => {
          self.drain_stack(self.vm.value_stack.len() - 1);
        }
        InstructionKind::Print => {
          let val = self.pop();
          println!("{}", val.repr(self.vm));
        }
        InstructionKind::JumpIfFalse => {
          if !bool::try_from(&self.pop())? {
            self.frame_mut().jmp(next_instruction.operand as usize);
          }
        }
        InstructionKind::JumpIfFalsePreserving => {
          if !bool::try_from(self.peek())? {
            self.frame_mut().jmp(next_instruction.operand as usize)
          }
        }
        InstructionKind::JumpIfTruePreserving => {
          if bool::try_from(self.peek())? {
            self.frame_mut().jmp(next_instruction.operand as usize)
          }
        }
        InstructionKind::Jmp => {
          self.frame_mut().jmp(next_instruction.operand as usize);
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
          let func_handle = self.load_const(next_instruction.operand as usize);

          let func = match func_handle {
            Value::Obj(handle) => {
              Obj::<Function>::downcast(handle, self.vm).expect("MakeClosure expects a function")
            }
            _ => panic!("MakeClosure expects a function"),
          };

          let mut upvalues: Vec<Root<UpValue>> = Vec::new();

          let upvalue_descs = func.borrow(self.vm).upvalues.clone();

          for (_, upvalue_desc) in upvalue_descs {
            match upvalue_desc {
              UpValueDescriptor::Local { parent_stack_pos } => {
                let absolute_stack_pos = self.frame().base + parent_stack_pos;
                let upval = self
                  .vm
                  .alloc_typed(UpValue::Open { absolute_stack_pos })
                  .as_root(self.vm);
                self.open_upvalues.push(upval.as_obj());
                upvalues.push(upval);
              }
              UpValueDescriptor::Recursive { parent_upvalue_pos } => {
                let upval = self.active_fn().upvalues[parent_upvalue_pos];
                upvalues.push(upval.as_root(self.vm));
              }
            }
          }

          let closure = self.vm.alloc(ObjData::Closure(Closure {
            func,
            upvalues: upvalues.iter().map(|u| u.as_obj()).collect(),
          }));

          for upval in upvalues {
            upval.free(self.vm);
          }

          self.push_value(Value::Obj(closure));
        }
        InstructionKind::Callq => {
          let nargs = next_instruction.operand as usize;
          let f_idx = self.vm.stack_top() - nargs - 1;
          let handle = match self.vm.value_stack[f_idx] {
            Value::Obj(obj) => obj,
            _ => return Err(RuntimeError::new("Expected function")),
          };

          let closure = Obj::<Closure>::downcast(handle, self.vm)
            .ok_or_else(|| RuntimeError::new("Call can only be called on a function"))?;

          let true_arity = closure.func(self.vm).arity;
          if nargs != true_arity {
            return Err(RuntimeError::new(
              format!(
                "Called function with wrong number of args, saw {nargs} expecting {true_arity}"
              )
              .as_str(),
            ));
          }

          self.call_stack.push(self.vm.alloc_frame(closure, f_idx));
        }
        InstructionKind::Return => {
          let returned_frame = match self.call_stack.pop() {
            Some(called) => called,
            None => {
              assert!(self.vm.value_stack.is_empty());
              assert!(self.frame().base == 0);
              return Ok(());
            }
          };
          let ret_val = self.pop();

          let leftover_stack_vals = self.vm.frame(returned_frame).closure.func(self.vm).arity + 1;

          assert!(
            self.vm.value_stack.len() >= self.vm.frame(returned_frame).base + leftover_stack_vals,
            "We got calling conventions wrong. Stack sized {} when at least expecting bp={} + (f,args...)={}.",
            self.vm.value_stack.len(),
            self.vm.frame(returned_frame).base,
            leftover_stack_vals,
          );

          self.drain_stack(self.vm.frame(returned_frame).base);
          self.push_value(ret_val);
        }
        InstructionKind::LoadUpValue => {
          let val = {
            let idx = next_instruction.operand as usize;
            match *self.upval(idx as usize).borrow(self.vm) {
              UpValue::Open { absolute_stack_pos } => self.vm.value_stack[absolute_stack_pos],
              UpValue::Closed(val) => val,
            }
          };

          self.push_value(val);
        }
        InstructionKind::SetUpValue => {
          let set_to = self.pop();

          let pos = {
            let upval = self.upval(next_instruction.operand as usize);
            match *upval.borrow(self.vm) {
              UpValue::Open { absolute_stack_pos } => Either::Left(absolute_stack_pos),
              UpValue::Closed(_) => Either::Right(upval),
            }
          };

          match pos {
            Either::Left(stack_pos) => self.vm.value_stack[stack_pos] = set_to,
            Either::Right(heap_val) => *heap_val.borrow_mut(self.vm) = UpValue::Closed(set_to),
          }
        }
        InstructionKind::PopUpValue => {
          self.drain_stack(self.vm.value_stack.len() - 1);
        }
      }
    }
  }
}
