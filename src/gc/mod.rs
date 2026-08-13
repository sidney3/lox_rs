mod block;
mod gc_ref;
mod header;
mod heap;
mod trace;

pub use gc_ref::{Ref, RefMut, UncheckedRef};
pub use heap::{AllocResult, Handle, Heap, HeapConfig};
pub use trace::{Trace, Tracer};
