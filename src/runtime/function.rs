use smallvec::SmallVec;

use super::Symbol;
use super::{ObjData, ObjKind};
use crate::asm::Chunk;

// Compile/runtime bridge: this gets thrown on the static
// function and consumed by the runtime (to resolve the
// actual runtime UpValues)
#[derive(Debug, Clone, Copy)]
pub enum UpValueDescriptor {
  // "parent" here refers to the parent of the closure
  Local { parent_stack_pos: usize },
  Recursive { parent_upvalue_pos: usize },
}

#[derive(Debug, Clone)]
pub struct Function {
  pub chunk: Box<Chunk>,
  pub arity: usize,
  pub name: Symbol,
  pub upvalues: Vec<UpValueDescriptor>,
}

impl ObjKind for Function {
  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::Func(s) => Some(s),
      _ => None,
    }
  }

  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::Func(s) => Some(s),
      _ => None,
    }
  }

  fn embed(self) -> ObjData {
    ObjData::Func(self)
  }
}
