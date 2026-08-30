use lasso::Key;

use super::error::{Error, Result};
use super::instruction::{Instruction, InstructionKind, OperandType};
use super::symbolic_instruction::{Label, SymbolicInstruction, SymbolicOp, SymbolicProgram};
use crate::obj::{Class, Closure, Function, Obj, UpValueDescriptor};
use crate::runtime::{Root, Runtime, Symbol, Value};

// Expression results are transient on the stack.
//
// The only persistent value on the stack is a global.
//
// The index of `Local` is exactly its position on the stack
// (relative to the stack pointer when the function is called)..
#[derive(Clone, Copy)]
pub struct Local {
  pub scope_depth: usize,
  pub symbol: Symbol,
}

enum VariableLocation {
  Global(Symbol),
  Local(usize),
  UpValue(usize), // upvalue array
}

pub struct FuncState {
  staging: Root<Function>,
  instructions: SymbolicProgram,
  locals: Vec<Local>,
  upvalues: Vec<(Symbol, UpValueDescriptor)>,
  scope_depth: usize,
  // active_loops.back() is the current loop
  active_loops: Vec<Label>,

  parent: Option<Box<Self>>,
}

pub struct FuncStack(Box<FuncState>);

impl FuncStack {
  pub fn new(main: FuncState) -> Self {
    Self(Box::new(main))
  }
  pub fn head_mut(&mut self) -> &mut Box<FuncState> {
    &mut self.0
  }

  pub fn push(&mut self, rt: &mut Runtime, name: Symbol, arity: usize) {
    let prev_head = std::mem::replace(&mut self.0, Box::new(FuncState::new(rt, name, arity, None)));
    self.0.set_parent(prev_head);
  }

  pub fn pop(&mut self, rt: &mut Runtime) -> Option<Root<Function>> {
    match self.0.try_pop() {
      Some(prev_head) => {
        let head = std::mem::replace(&mut self.0, prev_head);
        Some(head.assemble(rt))
      }
      None => None,
    }
  }

  pub fn pop_last(mut self, rt: &mut Runtime) -> Option<Root<Function>> {
    match self.0.try_pop() {
      Some(_) => None,
      None => Some(self.0.assemble(rt)),
    }
  }
}

impl FuncState {
  pub fn new(rt: &mut Runtime, name: Symbol, arity: usize, parent: Option<Box<Self>>) -> Self {
    let staging = rt.alloc_typed(Function::new(name, arity)).as_root(rt);
    Self {
      staging,
      instructions: SymbolicProgram::new(),
      locals: Vec::new(),
      upvalues: Vec::new(),
      active_loops: Vec::new(),
      scope_depth: 0,
      parent,
    }
  }

  fn emit_symbolic(&mut self, instruction: SymbolicInstruction) {
    self.instructions.add_instruction(instruction);
  }

  pub fn emit(&mut self, instruction: Instruction) {
    self
      .instructions
      .add_instruction(SymbolicInstruction::from_instruction(instruction));
  }

  // label_name is just for debugging, they don't have to be unique
  pub fn create_label(&mut self, label_name: &'static str) -> Label {
    self.instructions.create_label(label_name)
  }
  pub fn bind_label(&mut self, label: Label) {
    self.emit_symbolic(SymbolicInstruction::Label(label));
  }

  // Stack after: [methods..., class_def]
  pub fn make_class_instance(
    &mut self,
    rt: &mut Runtime,
    class: Obj<Class>,
    methods: &[Obj<Function>],
  ) -> Result<()> {
    for method in methods {
      let index = self.add_constant(rt, method.as_value())?;
      self.emit(Instruction {
        kind: InstructionKind::MakeClosure,
        operand: index,
      });
    }
    self.constant(rt, class.as_value())?;
    self.emit(Instruction::new(InstructionKind::InstantiateClass));

    Ok(())
  }
  // After these instructions, there will be a new symbol `called` that
  // refers to a closure instance of func.
  pub fn add_closure_to_scope(
    &mut self,
    rt: &mut Runtime,
    func: Obj<Function>,
    called: Symbol,
  ) -> Result<()> {
    let index = self.add_constant(rt, func.as_value())?;
    self.emit(Instruction {
      kind: InstructionKind::MakeClosure,
      operand: index,
    });

    // Acts on the top of the stack
    self.define_var(called)?;

    Ok(())
  }

  pub fn add_constant(&mut self, rt: &mut Runtime, constant: Value) -> Result<OperandType> {
    let constants = &mut self.staging.as_obj().borrow_mut(rt).constants;
    let index = constants.len();
    constants.push(constant);
    OperandType::try_from(index).map_err(|_| Error::IndexOutOfRange(index))
  }

  // These two functions exist only for the dance of safely maintaining the
  // state of the stack.
  //
  // PUSH is done by replacing the head stack node with a trivial FuncState,
  // and then set_parent'ing the previous head.
  //
  // POP is done by orphaning the head node, and replacing the previous head with it.
  fn try_pop(&mut self) -> Option<Box<Self>> {
    self.parent.take()
  }

  fn set_parent(&mut self, parent: Box<Self>) {
    self.parent = Some(parent);
  }

  pub fn assemble(self, rt: &mut Runtime) -> Root<Function> {
    let mut func = self.staging.as_obj().borrow_mut(rt);

    func.upvalues = self.upvalues;
    func.instructions = self
      .instructions
      .parse()
      .expect("Symbolic resolution failure.");

    self.staging
  }

  pub fn at_global_depth(&self) -> bool {
    self.scope_depth == 0
  }

  pub fn begin_loop(&mut self, end_label: Label) {
    self.active_loops.push(end_label);
  }

  pub fn end_loop(&mut self) {
    self
      .active_loops
      .pop()
      .expect("End loop called with no active loop");
  }

  pub fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  pub fn end_scope(&mut self) {
    assert!(!self.at_global_depth());

    while let Some(local) = self.locals.last() {
      if local.scope_depth != self.scope_depth {
        break;
      }

      let upval = self
        .upvalues
        .iter()
        .position(|upvalue| upvalue.0 == local.symbol)
        .map(|i| self.upvalues.remove(i));

      let instruction = if upval.is_some() {
        InstructionKind::PopUpValue
      } else {
        InstructionKind::Pop
      };
      self.emit(Instruction::new(instruction));
      self.locals.pop();
    }
    self.scope_depth -= 1;
  }

  //////////////////////////
  /// Instructions
  //////////////////////////
  // Stack before: [ assign_to, receiver ]
  pub fn set_class_attr(&mut self, _: &mut Runtime, attr: Symbol) -> Result<()> {
    self.emit(Instruction {
      kind: InstructionKind::SetClassAttribute,
      operand: index_to_op(attr.into_usize())?,
    });

    Ok(())
  }
  // Stack before: [ receiver ]
  pub fn get_class_attr(&mut self, _: &Runtime, attr: Symbol) -> Result<()> {
    self.emit(Instruction {
      kind: InstructionKind::LoadClassAttribute,
      operand: index_to_op(attr.into_usize())?,
    });

    Ok(())
  }
  pub fn pop(&mut self) {
    self.emit(Instruction::new(InstructionKind::Pop));
  }

  pub fn print(&mut self) {
    self.emit(Instruction::new(InstructionKind::Print));
  }

  pub fn assert(&mut self) {
    self.emit(Instruction::new(InstructionKind::Assert));
  }

  pub fn jmp(&mut self, to: Label) {
    self.emit_symbolic(SymbolicInstruction::Instruction(
      InstructionKind::Jmp,
      SymbolicOp::Label(to),
    ));
  }
  pub fn jmp_if_false(&mut self, to: Label) {
    self.emit_symbolic(SymbolicInstruction::Instruction(
      InstructionKind::JumpIfFalse,
      SymbolicOp::Label(to),
    ));
  }
  pub fn jmp_if_false_preserving(&mut self, to: Label) {
    self.emit_symbolic(SymbolicInstruction::Instruction(
      InstructionKind::JumpIfFalsePreserving,
      SymbolicOp::Label(to),
    ));
  }
  pub fn jmp_if_true_preserving(&mut self, to: Label) {
    self.emit_symbolic(SymbolicInstruction::Instruction(
      InstructionKind::JumpIfTruePreserving,
      SymbolicOp::Label(to),
    ));
  }

  pub fn constant(&mut self, rt: &mut Runtime, constant: Value) -> Result<()> {
    let index = self.add_constant(rt, constant)?;
    self.emit(Instruction {
      kind: InstructionKind::Constant,
      operand: index,
    });
    Ok(())
  }

  pub fn callq(&mut self, nargs: usize) -> Result<()> {
    self.emit(Instruction {
      kind: InstructionKind::Callq,
      operand: index_to_op(nargs)?,
    });
    Ok(())
  }

  pub fn ret(&mut self) {
    self.emit(Instruction::new(InstructionKind::Return));
  }

  pub fn bind_this(&mut self, to: Symbol) {
    self.emit(Instruction::new(InstructionKind::PushThis));
    self.add_local(to);
  }

  pub fn class(&mut self, rt: &mut Runtime, class: Obj<Class>) -> Result<()> {
    let const_index = self.add_constant(rt, class.as_value())?;

    self.emit(Instruction {
      kind: InstructionKind::InstantiateClass,
      operand: const_index,
    });

    Ok(())
  }

  pub fn loop_break(&mut self) -> Result<()> {
    if let Some(loop_end) = self.active_loops.last() {
      self.emit_symbolic(SymbolicInstruction::Instruction(
        InstructionKind::Jmp,
        SymbolicOp::Label(*loop_end),
      ));
      Ok(())
    } else {
      Err(Error::NakedBreak)
    }
  }

  // stack_offset is the number of positions BEFORE the stack pointer.
  // This is obviously pretty low level, and if you can avoid using it
  // you should.
  pub fn add_local(&mut self, sym: Symbol) {
    // assert!(!self.at_global_depth());
    self.locals.push(Local {
      scope_depth: self.scope_depth,
      symbol: sym,
    });
  }

  pub fn define_var(&mut self, sym: Symbol) -> Result<()> {
    if self.at_global_depth() {
      self.emit(Instruction {
        kind: InstructionKind::AddGlobal,
        operand: index_to_op(sym.into_usize())?,
      });
    } else {
      self.add_local(sym);
    }

    Ok(())
  }

  // mut as we late bind globals, so if this is a global this might be the first
  // time we see `var_name`
  fn find_local(&self, sym: Symbol) -> Option<usize> {
    let maybe_local_index = self
      .locals
      .iter()
      .enumerate()
      .rfind(|(_, l)| l.symbol == sym)
      .map(|(i, _)| i);

    if let Some(local_index) = maybe_local_index {
      Some(local_index)
    } else {
      None
    }
  }

  fn resolve_upvalue(&mut self, sym: Symbol) -> Option<UpValueDescriptor> {
    if let Some(local_index) = self.find_local(sym) {
      Some(UpValueDescriptor::Local {
        parent_stack_pos: local_index,
      })
    } else if let Some(recursive_upvalue) = self
      .parent
      .as_deref_mut()
      .and_then(|p| p.resolve_upvalue(sym))
    {
      Some(UpValueDescriptor::Recursive {
        parent_upvalue_pos: self.add_upvalue(sym, recursive_upvalue),
      })
    } else {
      None
    }
  }

  fn add_upvalue(&mut self, sym: Symbol, desc: UpValueDescriptor) -> usize {
    let maybe_upvalue_index = self
      .upvalues
      .iter()
      .enumerate()
      .rfind(|(_, l)| l.0 == sym)
      .map(|(i, _)| i);

    if let Some(upvalue_index) = maybe_upvalue_index {
      upvalue_index
    } else {
      let idx = self.upvalues.len();
      self.upvalues.push((sym, desc));
      idx
    }
  }

  // &mut as resolve_upvalue, in the recursive case, might have to register
  // the thing
  //
  // Never null as we fallback to VariableLocation::Global (late binding means
  // we don't really know if a global load is valid, we just emit and let
  // it fail at runtime :shrug:)
  fn resolve_var_location(&mut self, sym: Symbol) -> VariableLocation {
    if let Some(local_index) = self.find_local(sym) {
      VariableLocation::Local(local_index)
    } else if let Some(upvalue_descriptor) = self
      .parent
      .as_deref_mut()
      .and_then(|p| p.resolve_upvalue(sym))
    {
      VariableLocation::UpValue(self.add_upvalue(sym, upvalue_descriptor))
    } else {
      VariableLocation::Global(sym)
    }
  }

  // Find, at compile time, the symbol `sym` and push it
  // onto the stack.
  pub fn load_var(&mut self, sym: Symbol) -> Result<()> {
    match self.resolve_var_location(sym) {
      VariableLocation::Local(idx) => self.emit(Instruction {
        kind: InstructionKind::LoadLocal,
        operand: index_to_op(idx)?,
      }),
      VariableLocation::UpValue(upvalue_index) => self.emit(Instruction {
        kind: InstructionKind::LoadUpValue,
        operand: index_to_op(upvalue_index)?,
      }),
      VariableLocation::Global(sym) => self.emit(Instruction {
        kind: InstructionKind::LoadGlobal,
        operand: index_to_op(sym.into_usize())?,
      }),
    }
    Ok(())
  }

  // NB: as this is an expression, this emits
  // an additional instruction to push
  // the new value onto the stack
  pub fn set_variable(&mut self, lhs: Symbol) -> Result<()> {
    match self.resolve_var_location(lhs) {
      VariableLocation::Global(g) => {
        let global_index_op = index_to_op(g.into_usize())?;
        self.emit(Instruction {
          kind: InstructionKind::SetGlobal,
          operand: global_index_op,
        });
        self.emit(Instruction {
          kind: InstructionKind::LoadGlobal,
          operand: global_index_op,
        });
      }
      VariableLocation::Local(l) => {
        let local_index_op = index_to_op(l)?;
        self.emit(Instruction {
          kind: InstructionKind::SetLocal,
          operand: local_index_op,
        });
        self.emit(Instruction {
          kind: InstructionKind::LoadLocal,
          operand: local_index_op,
        });
      }
      VariableLocation::UpValue(l) => {
        let local_index_op = index_to_op(l)?;
        self.emit(Instruction {
          kind: InstructionKind::SetUpValue,
          operand: local_index_op,
        });
        self.emit(Instruction {
          kind: InstructionKind::LoadUpValue,
          operand: local_index_op,
        });
      }
    };
    Ok(())
  }
}

fn index_to_op(index: usize) -> Result<OperandType> {
  OperandType::try_from(index).map_err(|_| Error::IndexOutOfRange(index))
}
