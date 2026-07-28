use std::cell::Cell;

#[derive(Clone, Copy)]
pub enum BorrowState {
  Unborrowed,
  Shared(u32),
  BorrowedMut,
}

pub struct GcHeader {
  borrow_state: Cell<BorrowState>,
}

impl GcHeader {
  pub fn new() -> Self {
    Self {
      borrow_state: Cell::new(BorrowState::Unborrowed),
    }
  }

  // runtime borrow checker
  pub fn start_borrow(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::Unborrowed => BorrowState::Shared(1),
      BorrowState::Shared(k) => BorrowState::Shared(k + 1),
      BorrowState::BorrowedMut => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }

  pub fn end_borrow(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::Shared(k) if k == 1 => BorrowState::Unborrowed,
      BorrowState::Shared(k) if k > 1 => BorrowState::Shared(k - 1),
      _ => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }

  pub fn start_borrow_mut(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::Unborrowed => BorrowState::BorrowedMut,
      _ => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }

  pub fn end_borrow_mut(&self) {
    let borrow_state_after = match self.borrow_state.get() {
      BorrowState::BorrowedMut => BorrowState::Unborrowed,
      _ => panic!("unreachable"),
    };
    self.borrow_state.set(borrow_state_after);
  }
}
