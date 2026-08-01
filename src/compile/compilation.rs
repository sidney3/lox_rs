use crate::runtime::Function;

pub struct Compilation {
  pub main: Function,
  pub symbols: lasso::Rodeo,
}
