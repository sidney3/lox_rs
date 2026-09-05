use super::ObjData;
use crate::asm::Instruction;
use crate::gc::{Trace, Tracer};
use crate::runtime::{Symbol, Value};
use lox_derive::LoxObj;

// Compile/runtime bridge: this gets thrown on the static
// function and consumed by the runtime (to resolve the
// actual runtime UpValues)
#[derive(Debug, Clone, Copy)]
pub enum UpValueDescriptor {
  // "parent" here refers to the parent of the closure
  Local { parent_stack_pos: usize },
  Recursive { parent_upvalue_pos: usize },
}

#[derive(Debug, Clone, LoxObj)]
pub struct Function {
  pub arity: usize,
  pub name: Symbol,
  pub upvalues: Vec<(Symbol, UpValueDescriptor)>,
  pub instructions: Vec<Instruction>,
  pub constants: Vec<Value>,
}

impl Function {
  pub fn new(name: Symbol, arity: usize) -> Self {
    Self {
      instructions: Vec::new(),
      constants: Vec::new(),
      arity,
      name,
      upvalues: Vec::new(),
    }
  }
}

impl Trace<ObjData> for Function {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    for constant in &self.constants {
      constant.trace(tracer)
    }
  }
}
