use super::ObjData;
use super::ObjKind;
use super::Value;

#[derive(Debug)]
pub struct ValueObject {
  pub val: Value,
}

impl ObjKind for ValueObject {
  fn embed(self) -> ObjData {
    ObjData::Value(self)
  }
  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::Value(v) => Some(v),
      _ => None,
    }
  }
  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::Value(v) => Some(v),
      _ => None,
    }
  }
}
