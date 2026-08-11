use std::cell::Cell;
use strum::Display;

#[derive(Clone, Copy, Display)]
pub(super) enum BorrowState {
  Unborrowed,
  Shared(u32),
  BorrowedMut,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub(super) enum Marking {
  White,
  Black,
}

pub(super) struct GcHeader {
  borrow_state: Cell<BorrowState>,
  marking: Cell<Marking>,
}

impl GcHeader {
  pub(super) fn new() -> Self {
    Self {
      borrow_state: Cell::new(BorrowState::Unborrowed),
      marking: Cell::new(Marking::White),
    }
  }

  // Fine to use "proper" `&mut` because this gets mutated during
  // a collection cycle.
  pub(super) fn replace_mark(&mut self, next: Marking) -> Marking {
    self.marking.replace(next)
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
