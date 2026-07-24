use crate::asm::{Chunk, Instruction};

pub struct CallFrame<'c> {
  pub chunk: &'c Chunk,
  pub ip: usize,
  pub base: usize, // this frame's window starts at stack[base]
}

impl<'c> CallFrame<'c> {
  pub fn pop_instruction(&mut self) -> Option<Instruction> {
    match self.chunk.instructions.get(self.ip) {
      Some(&instruction) => {
        self.ip += 1;
        Some(instruction)
      }
      None => None,
    }
  }

  pub fn jmp(&mut self, to: usize) {
    assert!(to < self.chunk.instructions.len());

    self.ip = to;
  }
}
