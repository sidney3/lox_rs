mod block;
mod gc_ref;
mod header;
mod heap;

pub use gc_ref::{Ref, RefMut};
pub use heap::{Handle, Heap};
