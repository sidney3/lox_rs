use super::ObjData;
use crate::gc::{Trace, Tracer};
use lox_derive::LoxObj;

#[derive(LoxObj, Debug)]
pub struct LoxString {
  raw: String,
}

impl LoxString {
  pub fn new(s: String) -> Self {
    Self { raw: s }
  }
  pub fn as_str(&self) -> &str {
    self.raw.as_str()
  }
}

impl Trace<ObjData> for LoxString {
  fn trace(&self, _: &mut Tracer<ObjData>) {}
}
