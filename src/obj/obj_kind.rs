use super::ObjData;

pub trait ObjKind: Sized {
  fn project(obj: &ObjData) -> Option<&Self>;
  fn project_mut(obj: &mut ObjData) -> Option<&mut Self>;
  fn embed(self) -> ObjData;
}
