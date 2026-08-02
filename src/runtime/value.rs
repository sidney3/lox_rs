use super::RuntimeError;
use crate::runtime::{Handle, ObjData, Runtime};

pub type Num = f64;
pub type Bool = bool;

#[derive(Debug, Copy, Clone)]
pub enum Value {
  Num(Num),
  Obj(Handle),
  Bool(Bool),
  Nil,
}

impl TryFrom<&Value> for Bool {
  type Error = RuntimeError;

  fn try_from(value: &Value) -> Result<Self, Self::Error> {
    match value {
      &Value::Bool(b) => Ok(b),
      &Value::Num(x) => Ok(x != 0f64),
      Value::Obj(_) => Err(RuntimeError::new("Obj cannot be converted to bool")),
      Value::Nil => Ok(false),
    }
  }
}

// Value::add might produce a new `Object` and thus require a heap allocation.
// However, we'd really like `add` to be a pure function... Which this type
// allows us to model
pub enum ProtoValue {
  Imm(Value),
  Obj(ObjData), // to be put on the heap
}

impl ProtoValue {
  pub fn to_value(self, vm: &mut Runtime) -> Value {
    match self {
      Self::Imm(val) => val,
      Self::Obj(obj) => Value::Obj(vm.alloc(obj)),
    }
  }
}

impl Value {
  pub fn add(self, rhs: Value, vm: &Runtime) -> Result<ProtoValue, RuntimeError> {
    match (self, rhs) {
      (Value::Num(a), Value::Num(b)) => Ok(ProtoValue::Imm(Value::Num(a + b))),
      (Value::Obj(a), Value::Num(b)) => Ok(vm.borrow(a).add_right(b, vm)?),
      (Value::Num(a), Value::Obj(b)) => Ok(vm.borrow(b).add_left(a, vm)?),
      (Value::Obj(a), Value::Obj(b)) => {
        let lhs = vm.borrow(a);
        let rhs = vm.borrow(b);
        lhs.add(&rhs, vm)
      }
      _ => Err(RuntimeError {
        msg: format!("Bad binary expression between: {:?}, {:?}", self, rhs,),
      }),
    }
  }

  pub fn divide(self, rhs: Value, _: &Runtime) -> Result<Value, RuntimeError> {
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

  pub fn equals(self, rhs: Value, vm: &Runtime) -> Result<Value, RuntimeError> {
    let eq = match (self, rhs) {
      (Value::Num(a), Value::Num(b)) => a == b,
      (Value::Obj(a), Value::Obj(b)) => {
        let l = vm.borrow(a);
        let r = vm.borrow(b);

        l.equals(&r)?
      }
      (Value::Bool(a), Value::Bool(b)) => a == b,
      (Value::Nil, Value::Nil) => true,
      _ => {
        return Err(RuntimeError {
          msg: "Incompatible binary operators to ==".to_string(),
        });
      }
    };
    Ok(Value::Bool(eq))
  }
  pub fn neq(self, rhs: Value, vm: &Runtime) -> Result<Value, RuntimeError> {
    match self.equals(rhs, vm)? {
      Value::Bool(b) => Ok(Value::Bool(!b)),
      _ => panic!("unreachable. equals() should always return Value::Bool"),
    }
  }
  pub fn mult(self, rhs: Value, _: &Runtime) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Num(a * b), "*")
  }

  pub fn sub(self, rhs: Value, _: &Runtime) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Num(a - b), "-")
  }

  pub fn greater(self, rhs: Value, _: &Runtime) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Bool(a > b), ">")
  }

  pub fn less(self, rhs: Value, _: &Runtime) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Bool(a < b), "<")
  }
  pub fn leq(self, rhs: Value, _: &Runtime) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Bool(a <= b), "<=")
  }
  pub fn geq(self, rhs: Value, _: &Runtime) -> Result<Value, RuntimeError> {
    self.binary_arithmetic(rhs, |a, b| Value::Bool(a >= b), ">=")
  }

  pub fn not(self, _: &Runtime) -> Result<Value, RuntimeError> {
    match self {
      Value::Bool(b) => Ok(Value::Bool(!b)),
      _ => Err(RuntimeError::new("Invalid input to not")),
    }
  }

  pub fn unary_minus(self, _: &Runtime) -> Result<Value, RuntimeError> {
    match self {
      Value::Num(x) => Ok(Value::Num(-x)),
      _ => Err(RuntimeError::new("Invalid input to minus")),
    }
  }

  pub fn repr(self, vm: &Runtime) -> String {
    match self {
      Value::Num(x) => x.to_string(),
      Value::Obj(o) => {
        format!("{}", *vm.borrow(o))
      }
      Value::Bool(b) => if b { "True" } else { "False" }.to_string(),
      Value::Nil => "nil".to_string(),
    }
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
