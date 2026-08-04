use super::ObjData;
use super::ObjKind;
use super::Value;

#[derive(Debug)]
pub enum UpValue {
  Open { absolute_stack_pos: usize },
  Closed(Value),
}

impl ObjKind for UpValue {
  fn embed(self) -> ObjData {
    ObjData::UpValue(self)
  }

  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::UpValue(v) => Some(v),
      _ => None,
    }
  }

  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::UpValue(v) => Some(v),
      _ => None,
    }
  }
}
