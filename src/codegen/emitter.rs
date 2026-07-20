use lasso::Key;

use super::Compilation;
use super::chunk::Chunk;
use super::constant::Constant;
use super::error::{Error, Result};
use super::instruction::{Instruction, InstructionKind, OperandType};
use super::symbolic_instruction::{SymbolicInstruction, SymbolicProgram};

pub type Symbol = lasso::Spur;

// Expression results are transient on the stack.
//
// The only persistent value on the stack is a global.
//
// The index of `Local` is exactly its position on the stack
// (relative to the base).
#[derive(Clone, Copy)]
pub struct Local {
  pub scope_depth: usize,
  pub symbol: Symbol,
}

type LValue<'a> = &'a str;

enum VariableLocation {
  Global(Symbol),
  Local(usize),
}

pub struct Emitter {
  constants: Vec<Constant>,
  instructions: SymbolicProgram,
  locals: Vec<Local>,
  scope_depth: usize,

  // names that survive to runtime, as strings.
  // NB: the runtime will peer into this arena!
  names: lasso::Rodeo,
}

impl Emitter {
  pub fn new() -> Self {
    Self {
      constants: Vec::new(),
      instructions: SymbolicProgram::new(),
      names: lasso::Rodeo::new(),
      locals: Vec::new(),
      scope_depth: 0,
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

  pub fn emit_constant(&mut self, constant: Constant) -> Result<()> {
    let index = self.add_constant(constant)?;
    self.emit(Instruction {
      kind: InstructionKind::Constant,
      operand: index,
    });
    Ok(())
  }

  pub fn add_constant(&mut self, constant: Constant) -> Result<OperandType> {
    let index = self.constants.len();
    self.constants.push(constant);
    OperandType::try_from(index).map_err(|_| Error::IndexOutOfRange(index))
  }

  pub fn finalize(self) -> Compilation {
    Compilation {
      chunk: Chunk {
        instructions: self
          .instructions
          .parse()
          .expect("Symbolic resolution failure."),
      },
      symbols: self.names,
      constants: self.constants,
    }
  }

  pub fn at_global_depth(&self) -> bool {
    self.scope_depth == 0
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

  pub fn emit_create_var<F>(&mut self, var_name: &str, create_with: F) -> Result<()>
  where
    F: FnOnce(&mut Self) -> Result<()>,
  {
    let sym = self.names.get_or_intern(var_name);
    create_with(self)?;
    if self.at_global_depth() {
      self.emit(Instruction {
        kind: InstructionKind::AddGlobal,
        operand: index_to_op(sym.into_usize())?,
      });
    } else {
      self.locals.push(Local {
        scope_depth: self.scope_depth,
        symbol: sym,
      });
    }

    Ok(())
  }

  // mut as we late bind globals, so if this is a global this might be the first
  // time we see `var_name`
  fn find_var(&mut self, var_name: LValue) -> VariableLocation {
    let sym = self.names.get_or_intern(var_name);
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

  pub fn emit_load_var(&mut self, var_name: LValue) -> Result<()> {
    match self.find_var(var_name) {
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
  pub fn emit_set_variable<F>(&mut self, lhs: LValue, emit_rhs: F) -> Result<()>
  where
    F: FnOnce(&mut Self) -> Result<()>,
  {
    emit_rhs(self)?;

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
