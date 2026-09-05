use crate::runtime::{Runtime, RuntimeError, Value};

pub(super) fn print(rt: &Runtime, vals: &[Value]) -> Result<Value, RuntimeError> {
  match vals {
    [fst] => {
      println!("{}", fst.repr(rt));
      Ok(Value::Nil)
    }
    _ => Err(RuntimeError::new("Print takes 1 arg")),
  }
}
