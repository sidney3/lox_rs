use std::ops::Deref;

use lasso::Key;
use log::{debug, info, warn};
use nonempty::{NonEmpty, nonempty};
use smallvec::SmallVec;

use crate::asm::{Instruction, InstructionKind};
use crate::gc::Ref;
use crate::obj::{
  BoundMethod, Class, Closure, Function, Instance, NativeFunction, Obj, ObjData, TryAsObjExt,
  UpValue, UpValueDescriptor,
};
use crate::runtime::{CallFrame, Callee, FrameIndex, Root, Runtime, RuntimeError, Symbol, Value};
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

    let main_frame = vm.alloc_frame(Callee::Func(main_closure.as_obj()), 0);

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

  fn try_load_this(&self) -> Option<Obj<Instance>> {
    Obj::<BoundMethod>::try_from_value(self.vm, self.load_stack(0))
      .map(|bound| bound.borrow(self.vm).receiver())
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
      .callee
      .as_func(self.vm)
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
    self.frame().callee.as_closure(self.vm)
  }

  fn upval(&self, idx: usize) -> Obj<UpValue> {
    self.active_fn().upvalues[idx]
  }

  fn push_value(&mut self, val: Value) {
    self.vm.value_stack.push(val);
  }

  fn call_native(&mut self, f: Obj<NativeFunction>, nargs: usize) -> Result<Value, RuntimeError> {
    let until = self.vm.value_stack.len() - nargs;
    let args: Vec<Value> = self.vm.value_stack.drain(until..).collect();

    f.borrow(self.vm).call(self.vm, &args)
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

  fn make_closure(&mut self, f: Obj<Function>) -> Root<Closure> {
    let mut upvalues: Vec<Root<UpValue>> = Vec::new();

    let upvalue_descs = f.borrow(self.vm).upvalues.clone();

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

    let closure = self
      .vm
      .alloc_typed(Closure {
        func: f,
        upvalues: upvalues.iter().map(|u| u.as_obj()).collect(),
      })
      .as_root(self.vm);

    for upval in upvalues {
      upval.free(self.vm);
    }

    closure
  }

  // Doesn't call the constructor - caller needs to take care of this.
  fn make_instance(&mut self, class: Obj<Class>) -> Root<Instance> {
    let (name, methods) = {
      let x = class.borrow(self.vm);

      (x.symbol(), x.methods().clone())
    };

    let instance = self.vm.alloc_typed(Instance::new(name)).as_root(self.vm);

    for f in methods {
      let bound_method = self.vm.alloc_typed(BoundMethod::new(instance.as_obj(), f));
      instance
        .as_obj()
        .borrow_mut(self.vm)
        .add_method(bound_method);
    }

    instance
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
        InstructionKind::Equals => self.execute_binary(Value::equals_value)?,
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
        InstructionKind::AddGlobal => {
          let assign = self.pop();
          let global_idx =
            Symbol::try_from_usize(next_instruction.operand as usize).expect("Bad spur");

          self.vm.globals.insert(global_idx, assign);
        }
        InstructionKind::LoadGlobal => {
          let global_idx =
            Symbol::try_from_usize(next_instruction.operand as usize).expect("Bad spur");

          let global = self.vm.globals.get(&global_idx).ok_or_else(|| {
            RuntimeError::new(
              format!("Unrecognized ident: {}", self.vm.resolve_sym(global_idx)).as_str(),
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
            Symbol::try_from_usize(next_instruction.operand as usize).expect("Bad spur");

          match self.vm.globals.get_mut(&global_idx) {
            Some(global) => {
              *global = assign;
            }
            None => {
              let ident = self.vm.resolve_sym(global_idx);

              return Err(RuntimeError::new(
                format!("Unrecognized ident: {}", ident).as_str(),
              ));
            }
          }
        }

        InstructionKind::MakeClosure => {
          let func = self
            .load_const(next_instruction.operand as usize)
            .try_as_obj(self.vm)
            .expect("MakeClosure expects a function");

          let closure = self.make_closure(func);
          self.push_value(closure.as_value());
          closure.free(self.vm);
        }
        InstructionKind::Callq => {
          let nargs = next_instruction.operand as usize;
          let f_idx = self.vm.stack_top() - nargs - 1;

          let callee = {
            let val = self.vm.value_stack[f_idx];

            if let Some(closure) = Obj::<Closure>::try_from_value(self.vm, val) {
              Ok(Callee::Func(closure))
            } else if let Some(bound_method) = Obj::<BoundMethod>::try_from_value(self.vm, val) {
              Ok(Callee::Method(bound_method))
            } else if let Some(class_def) = Obj::<Class>::try_from_value(self.vm, val) {
              let instance = self.make_instance(class_def);
              let constructor: Obj<BoundMethod> = instance
                .as_obj()
                .borrow(self.vm)
                .load_attr(self.vm, self.vm.init_sym())
                .expect("All classes must have constructors")
                .try_as_obj(self.vm)
                .expect("All constructors must be BoundMethods");

              // Kind of evil? We want to decide to invoke a DIFFERENT callable
              // (the constructor) inside the runtime. Python for handles this by
              // allowing recursive calls. We just lie and overwrite the called object
              // with what we want called (the constructor).
              self.vm.value_stack[f_idx] = constructor.as_value();
              instance.free(self.vm);
              Ok(Callee::Class(constructor))
            } else if let Some(native) = Obj::<NativeFunction>::try_from_value(self.vm, val) {
              // subvert the normal calling convention
              self.call_native(native, nargs)?;
              continue;
            } else {
              let msg = format!(
                "Call can only be called on a function. Called on: {:?}",
                val
              );
              Err(RuntimeError::new(msg.as_str()))
            }
          }?;

          let true_arity = callee.as_func(self.vm).arity;
          if nargs != true_arity {
            return Err(RuntimeError::new(
              format!(
                "Called function with wrong number of args, saw {nargs} expecting {true_arity}"
              )
              .as_str(),
            ));
          }

          self.call_stack.push(self.vm.alloc_frame(callee, f_idx));
        }
        InstructionKind::Return => {
          let returned_frame = match self.call_stack.pop() {
            Some(called) => called,
            None => {
              for val in &self.vm.value_stack {
                if let &Value::Obj(o) = val {
                  warn!(
                    "Extra object left on stack: {:?}",
                    self.vm.borrow(o).deref()
                  )
                }
              }
              assert!(self.vm.value_stack.is_empty());
              assert!(self.frame().base == 0);
              return Ok(());
            }
          };
          let ret_val = self.pop();

          let leftover_stack_vals = self.vm.frame(returned_frame).callee.as_func(self.vm).arity + 1;

          assert!(
            self.vm.value_stack.len() >= self.vm.frame(returned_frame).base + leftover_stack_vals,
            "We got calling conventions wrong. Stack sized {} when at least expecting bp={} + (f,args...)={}.",
            self.vm.value_stack.len(),
            self.vm.frame(returned_frame).base,
            leftover_stack_vals,
          );

          self.drain_stack(self.vm.frame(returned_frame).base);

          match self.vm.frame(returned_frame).callee {
            Callee::Class(init) => {
              if ret_val != Value::Nil {
                return Err(RuntimeError::new("Constructor returned non-Nil"));
              }

              let instance = init.borrow(self.vm).receiver().as_value();
              self.push_value(instance);
            }
            _ => {
              self.push_value(ret_val);
            }
          }
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

        InstructionKind::MakeClass => {
          let name = Symbol::try_from_usize(next_instruction.operand as usize)
            .expect("Bad attribute access");
          let class = self.vm.alloc_typed(Class::new(name));
          self.push_value(class.as_value());
        }

        // [..., Class, Closure]
        InstructionKind::AddMethod => {
          let method: Obj<Function> = self.pop().try_as_obj(self.vm).expect("AddMethod");
          let class: Obj<Class> = self.pop().try_as_obj(self.vm).expect("AddMethod");

          let f = self.make_closure(method);
          class.borrow_mut(self.vm).add_method(self.vm, f.as_obj());
          f.free(self.vm);
          self.push_value(class.as_value());
        }

        // [..., Superclass, Subclass ]
        InstructionKind::Inherit => {
          let subclass: Obj<Class> = self.pop().try_as_obj(self.vm).expect("Subclass");
          let superclass: Obj<Class> = self
            .peek()
            .try_as_obj(self.vm)
            .ok_or_else(|| RuntimeError::new("SuperClass must be a class"))?;

          // copy-down inheritance
          for f in superclass.borrow(self.vm).methods().iter().copied() {
            subclass.borrow_mut(self.vm).add_method(self.vm, f);
          }
        }

        InstructionKind::LoadClassAttribute => {
          let maybe_instance = *self.peek();

          let attribute = {
            let class_instance = Obj::<Instance>::try_from_value(self.vm, maybe_instance)
              .ok_or_else(|| RuntimeError::new("TypeError: expected class"))?
              .borrow(self.vm);
            let symbol = Symbol::try_from_usize(next_instruction.operand as usize)
              .expect("Bad attribute access");
            if let Some(val) = class_instance.load_attr(self.vm, symbol) {
              val
            } else {
              return Err(RuntimeError::new(
                format!(
                  "Instance {} has no attribute {}",
                  class_instance.name(self.vm),
                  self.vm.resolve_sym(symbol)
                )
                .as_str(),
              ));
            }
          };

          self.pop();
          self.push_value(attribute);
        }
        InstructionKind::SetClassAttribute => {
          let maybe_instance = self.pop();

          // x = y is an expression which should leave y
          // on the stack. So we just leave it there...
          let assign_to = *self.peek();
          let symbol = Symbol::try_from_usize(next_instruction.operand as usize)
            .expect("Bad attribute access");
          Obj::<Instance>::try_from_value(self.vm, maybe_instance)
            .ok_or_else(|| RuntimeError::new("TypeError: expected class"))?
            .borrow_mut(self.vm)
            .set_attr(self.vm, symbol, assign_to)?;
        }
        InstructionKind::PushThis => {
          self.push_value(
            self
              .try_load_this()
              .expect("PushThis should only be called from within a class method")
              .as_value(),
          );
        }
        InstructionKind::LoadSuperMethod => {
          let super_: Obj<Class> = self.pop().try_as_obj(self.vm).expect("LoadSuperMethod");
          let this: Obj<Instance> = self.pop().try_as_obj(self.vm).expect("LoadSuperMethod");
          let property =
            Symbol::try_from_usize(next_instruction.operand as usize).expect("LoadSuperMethod");

          let method = match super_.borrow(self.vm).load_method(self.vm, property) {
            Some(x) => x,
            None => {
              return Err(RuntimeError::new(
                format!(
                  "Class {} has no method {}",
                  super_.borrow(self.vm).symbol().as_str(self.vm),
                  property.as_str(self.vm)
                )
                .as_str(),
              ));
            }
          };

          let bound = self.vm.alloc_typed(BoundMethod::new(this, method));
          self.push_value(bound.as_value());
        }
      }
    }
  }
}
