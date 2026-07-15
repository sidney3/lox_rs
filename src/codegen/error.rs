#[derive(Debug)]
pub enum Error {
  ConstantIndexOutOfRange(usize),
  GlobalIndexOutOfRange(usize),
}

pub type Result<T> = std::result::Result<T, Error>;
