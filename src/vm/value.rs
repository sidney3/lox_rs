use super::vm::Vm;

#[derive(Debug)]
pub enum Value {
    Num(f64),
}

impl Value {
    // TODO: this should also take in some
    // sort of "add reference" function
    pub fn clone(&mut self) -> Self {
        match self {
            Self::Num(x) => Self::Num(*x),
        }
    }
}
