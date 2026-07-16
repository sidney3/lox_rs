use crate::codegen::compile::compile_ast;
use crate::frontend::ast::Ast;

use super::Compilation;
use super::chunk::Chunk;
use super::constant::Constant;
use super::error::{Error, Result};
use super::instruction::{Instruction, InstructionKind, OperandType};
use lasso::Key;

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

pub struct Emitter {
  constants: Vec<Constant>,
  instructions: Vec<Instruction>,
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
      instructions: Vec::new(),
      names: lasso::Rodeo::new(),
      locals: Vec::new(),
      scope_depth: 0,
    }
  }

  pub fn emit(&mut self, instruction: Instruction) {
    self.instructions.push(instruction);
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
  pub fn get_or_intern_name(&mut self, name: &str) -> Result<OperandType> {
    index_to_op(self.names.get_or_intern(name).into_usize())
  }

  pub fn finalize(self) -> Compilation {
    Compilation {
      chunk: Chunk {
        instructions: self.instructions,
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

  pub fn emit_load_var(&mut self, var_name: &str) -> Result<()> {
    let sym = self.names.get_or_intern(var_name);
    let maybe_local_index = self
      .locals
      .iter()
      .enumerate()
      .rfind(|(_, l)| l.symbol == sym)
      .map(|(i, _)| i);

    if let Some(local_index) = maybe_local_index {
      self.emit(Instruction {
        kind: InstructionKind::LoadLocal,
        operand: index_to_op(local_index)?,
      })
    } else {
      self.emit(Instruction {
        kind: InstructionKind::LoadGlobal,
        operand: index_to_op(sym.into_usize())?,
      })
    }

    Ok(())
  }
}

fn index_to_op(index: usize) -> Result<OperandType> {
  OperandType::try_from(index).map_err(|_| Error::IndexOutOfRange(index))
}
