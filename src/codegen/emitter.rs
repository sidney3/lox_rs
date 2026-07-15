use crate::codegen::compile::compile_ast;
use crate::frontend::ast::Ast;

use super::Compilation;
use super::chunk::Chunk;
use super::constant::Constant;
use super::error::{Error, Result};
use super::instruction::{Instruction, InstructionKind, OperandType};
use lasso::Key;

#[derive(Clone, Copy)]
pub struct Local {
  pub depth: usize,
  pub ident: lasso::Spur,
}

pub struct Emitter {
  constants: Vec<Constant>,
  instructions: Vec<Instruction>,
  locals: Vec<Local>,
  depth: usize,

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
      depth: 0,
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
    OperandType::try_from(index).map_err(|_| Error::ConstantIndexOutOfRange(index))
  }

  pub fn get_or_intern_name(&mut self, name: &str) -> Result<OperandType> {
    let index = self.names.get_or_intern(name).into_usize();
    OperandType::try_from(index).map_err(|_| Error::GlobalIndexOutOfRange(index))
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

  pub fn depth(&self) -> usize {
    self.depth
  }

  pub fn at_global_depth(&self) -> bool {
    self.depth() == 0
  }

  pub fn begin_scope(&mut self) {
    self.depth += 1;
  }

  pub fn end_scope(&mut self) {
    assert!(!self.at_global_depth());

    while let Some(local) = self.locals.last() {
      if local.depth == self.depth {
        self.emit(Instruction::new(InstructionKind::Pop));
        self.locals.pop();
      }
    }
    self.depth -= 1;
  }
}
