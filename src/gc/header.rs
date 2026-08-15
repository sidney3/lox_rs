use std::cell::Cell;
use strum::Display;

#[derive(Clone, Copy, Display)]
pub(super) enum BorrowState {
  Unborrowed,
  Shared(u32),
  BorrowedMut,
}

pub(super) struct GcHeader {
  borrow_state: Cell<BorrowState>,
  generation: Cell<usize>,
}

impl GcHeader {
  pub(super) fn new(cur_generation: usize) -> Self {
    Self {
      borrow_state: Cell::new(BorrowState::Unborrowed),
      generation: Cell::new(cur_generation),
    }
  }

  // Mark a node as reachable. Returns if this is the first
  // time marking it (e.g. it needs to be explored)
  pub(super) fn mark(&self, collecting_generation: usize) -> bool {
    self.generation.replace(collecting_generation + 1) == collecting_generation
  }

  pub(super) fn valid_until(&self) -> usize {
    self.generation.get()
  }

  // runtime borrow checker
  pub(super) fn start_borrow(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::Unborrowed => BorrowState::Shared(1),
      BorrowState::Shared(k) => BorrowState::Shared(k + 1),
      BorrowState::BorrowedMut => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }

  pub(super) fn end_borrow(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::Shared(1) => BorrowState::Unborrowed,
      BorrowState::Shared(k) if k > 1 => BorrowState::Shared(k - 1),
      _ => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }

  pub(super) fn start_borrow_mut(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::Unborrowed => BorrowState::BorrowedMut,
      _ => panic!("Cannot borrow_mut. Object is: {}", self.borrow_state.get()),
    };
    self.borrow_state.set(borrow_state_after);
  }

  pub(super) fn end_borrow_mut(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::BorrowedMut => BorrowState::Unborrowed,
      _ => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }
}
