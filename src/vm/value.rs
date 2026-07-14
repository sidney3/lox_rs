use std::fmt;

use super::RuntimeError;
use super::vm::{Handle, Vm};

pub type Num = f64;

#[derive(Debug, Copy, Clone)]
pub enum Value {
  Num(Num),
  Obj(Handle),
  Bool(bool),
}

impl Value {
  pub fn add(self, rhs: Value, vm: &mut Vm) -> Result<Value, RuntimeError> {
    match (self, rhs) {
      (Value::Num(a), Value::Num(b)) => Ok(Value::Num(a + b)),
      (Value::Obj(a), Value::Num(b)) => {
        let obj = vm.obj(a).add_right(b)?;
        Ok(vm.alloc(obj))
      }
      (Value::Num(a), Value::Obj(b)) => {
        let obj = vm.obj(b).add_left(a)?;
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

  pub fn equals(self, rhs: Value, vm: &mut Vm) -> Result<Value, RuntimeError> {
    let eq = match (self, rhs) {
      (Value::Num(a), Value::Num(b)) => a == b,
      (Value::Obj(a), Value::Obj(b)) => {
        let l = vm.obj(a);
        let r = vm.obj(b);

        l.equals(&r)
      }
      (Value::Bool(a), Value::Bool(b)) => a == b,
      _ => {
        return Err(RuntimeError {
          msg: "Incompatible binary operators to ==".to_string(),
        });
      }
    };
    Ok(Value::Bool(eq))
  }
  pub fn mult(self, rhs: Value, _: &mut Vm) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Num(a * b), "*")
  }

  pub fn sub(self, rhs: Value, _: &mut Vm) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Num(a - b), "-")
  }

  pub fn greater(self, rhs: Value, _: &mut Vm) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Bool(a > b), ">")
  }

  pub fn less(self, rhs: Value, _: &mut Vm) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Bool(a < b), "<")
  }
  pub fn leq(self, rhs: Value, _: &mut Vm) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Bool(a <= b), "<=")
  }
  pub fn geq(self, rhs: Value, _: &mut Vm) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Bool(a >= b), ">=")
  }

  fn binary_arithmetic<F: FnOnce(Num, Num) -> Value>(
    self,
    rhs: Value,
    op: F,
    op_name: &str,
  ) -> Result<Value, RuntimeError> {
    match (self, rhs) {
      (Value::Num(a), Value::Num(b)) => Ok(op(a, b)),
      _ => Err(RuntimeError {
        msg: format!(
          "Incompatible binary expression: {:?} {op_name} {:?}",
          self, rhs
        ),
      }),
    }
  }
}
