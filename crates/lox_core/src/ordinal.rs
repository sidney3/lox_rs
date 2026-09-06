use std::fmt::{Debug, Display};
use std::hash::Hash;

/// An ordinal describes a finite type with a unique integer
/// representation (and thus a _finite_ integer representation)
///
/// We use it to model our enumeration types (which are much
/// cheaper to pass around and describe than the full types)
pub trait Ordinal: Debug + Hash + Eq + Clone + Copy + PartialEq + Display {
  const COUNT: usize;
  fn ord(&self) -> usize;
}
