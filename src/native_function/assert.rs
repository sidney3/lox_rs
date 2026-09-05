use crate::runtime::{Runtime, RuntimeError, Value};

pub(super) fn assert(_: &Runtime, vals: &[Value]) -> Result<Value, RuntimeError> {
  let operand: bool = match vals {
    [Value::Bool(b)] => *b,
    _ => return Err(RuntimeError::new("Assert takes two args")),
  };

  if operand {
    Ok(Value::Nil)
  } else {
    Err(RuntimeError::new("Assert failed"))
  }
}
