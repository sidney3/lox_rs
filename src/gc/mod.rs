mod block;
mod gc_ref;
mod header;
mod heap;
mod trace;

pub use gc_ref::{Ref, RefMut};
pub use heap::{Handle, Heap};
pub use trace::{Trace, Tracer};

