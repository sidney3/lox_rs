use super::Symbol;
use super::{ObjData, ObjKind};
use crate::asm::Chunk;

#[derive(Debug, Clone)]
pub struct Function {
  pub chunk: Box<Chunk>,
  pub arity: usize,
  pub name: Symbol,
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
