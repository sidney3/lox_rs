use lasso::Key;

use super::chunk::Chunk;
use super::error::{Error, Result};
use super::instruction::{Instruction, InstructionKind, OperandType};
use super::symbolic_instruction::{Label, SymbolicInstruction, SymbolicOp, SymbolicProgram};
use crate::runtime::{Function, Obj, ObjKind, Symbol, Value};

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
}

pub struct FuncState {
  constants: Vec<Value>,
  instructions: SymbolicProgram,
  locals: Vec<Local>,
  scope_depth: usize,
  // active_loops.back() is the current loop
  active_loops: Vec<Label>,
  name: Symbol,
  arity: usize,
}

impl FuncState {
  pub fn new(name: Symbol, arity: usize) -> Self {
    Self {
      constants: Vec::new(),
      instructions: SymbolicProgram::new(),
      locals: Vec::new(),
      active_loops: Vec::new(),
      scope_depth: 0,
      name,
      arity,
    }
  }

  pub fn emit_symbolic(&mut self, instruction: SymbolicInstruction) {
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

  pub fn add_constant(&mut self, constant: Value) -> Result<OperandType> {
    let index = self.constants.len();
    self.constants.push(constant);
    OperandType::try_from(index).map_err(|_| Error::IndexOutOfRange(index))
  }

  pub fn assemble(self) -> Function {
    let chunk = Box::new(Chunk {
      instructions: self
        .instructions
        .parse()
        .expect("Symbolic resolution failure."),
      constants: self.constants,
    });

    Function {
      chunk,
      arity: self.arity,
      name: self.name,
    }
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
      if local.scope_depth == self.scope_depth {
        self.emit(Instruction::new(InstructionKind::Pop));
        self.locals.pop();
      } else {
        break;
      }
    }
    self.scope_depth -= 1;
  }

  //////////////////////////
  /// Instructions
  //////////////////////////
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
  pub fn constant(&mut self, constant: Value) -> Result<()> {
    let index = self.add_constant(constant)?;
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

  pub fn trivial_ret(&mut self) -> Result<()> {
    self.constant(Value::Nil)?;
    self.emit(Instruction::new(InstructionKind::Return));
    Ok(())
  }
  pub fn ret(&mut self) {
    self.emit(Instruction::new(InstructionKind::Return));
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
    self.locals.push(Local {
      scope_depth: self.scope_depth,
      symbol: sym,
    });
  }

  pub fn make_closure(&mut self, func: Value) -> Result<()> {
    let func_index = self.add_constant(func)?;
    self.emit(Instruction {
      kind: InstructionKind::MakeClosure,
      operand: func_index,
    });
    Ok(())
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
  fn find_var(&mut self, sym: Symbol) -> VariableLocation {
    let maybe_local_index = self
      .locals
      .iter()
      .enumerate()
      .rfind(|(_, l)| l.symbol == sym)
      .map(|(i, _)| i);

    if let Some(local_index) = maybe_local_index {
      VariableLocation::Local(local_index)
    } else {
      VariableLocation::Global(sym)
    }
  }

  // Find, at compile time, the symbol `sym` and push it
  // onto the stack.
  pub fn load_var(&mut self, sym: Symbol) -> Result<()> {
    match self.find_var(sym) {
      VariableLocation::Local(idx) => self.emit(Instruction {
        kind: InstructionKind::LoadLocal,
        operand: index_to_op(idx)?,
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
    match self.find_var(lhs) {
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
    };
    Ok(())
  }
}

fn index_to_op(index: usize) -> Result<OperandType> {
  OperandType::try_from(index).map_err(|_| Error::IndexOutOfRange(index))
}
