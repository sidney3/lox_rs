use std::fmt;

use super::RuntimeError;
use super::vm::{Handle, Vm};

pub type Num = f64;

#[derive(Debug, Copy, Clone)]
pub enum Value {
  Num(Num),
  Obj(Handle),
}

impl Value {
  pub fn add(self, rhs: Value, vm: &mut Vm) -> Result<Value, RuntimeError> {
    match (self, rhs) {
      (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
      (Value::Obj(a), Value::Num(b)) => {
        let obj = vm.obj(a).add_left(b)?;
        Ok(vm.alloc(obj))
      }
      (Value::Num(a), Value::Obj(b)) => {
        let obj = vm.obj(b).add_right(a)?;
        Ok(vm.alloc(obj))
      }
      _ => Err(RuntimeError {
        msg: format!("Bad binary expression between: {:?}, {:?}", self, rhs,),
      }),
    }
  }

  pub fn divide(self, rhs: Value, _: &mut Vm) -> Result<Value, RuntimeError> {
    match (self, rhs) {
      (Value::Num(a), Value::Num(b)) => {
        if b != 0f64 {
          Ok(Value::Num(a / b))
        } else {
          Err(RuntimeError {
            msg: "Divide by zero".to_string(),
          })
        }
      }
      _ => Err(RuntimeError {
        msg: format!("Bad divide between: {:?}, {:?}", self, rhs),
      }),
    }
  }

  pub fn mult(self, rhs: Value, _: &mut Vm) -> Result<Value, RuntimeError> {
    match (self, rhs) {
      (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a * b)),
      _ => Err(RuntimeError {
        msg: format!("Bad mult between: {:?}, {:?}", self, rhs),
      }),
    }
  }

  pub fn sub(self, rhs: Value, _: &mut Vm) -> Result<Value, RuntimeError> {
    match (self, rhs) {
      (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a - b)),
      _ => Err(RuntimeError {
        msg: format!("Bad sub between: {:?}, {:?}", self, rhs),
      }),
    }
  }
}
