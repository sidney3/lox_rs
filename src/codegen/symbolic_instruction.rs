use std::collections::HashMap;

use super::{Instruction, InstructionKind, OperandType};

use log::warn;

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct Label {
  id: usize,
  name: &'static str,
}

#[derive(Clone, Copy)]
pub enum SymbolicOp {
  None,
  Imm(OperandType),
  Label(Label),
}

#[derive(Clone, Copy)]
pub enum SymbolicInstruction {
  Label(Label),
  Instruction(InstructionKind, SymbolicOp),
}

pub struct SymbolicProgram {
  instructions: Vec<SymbolicInstruction>,
  next_label: usize,
}

impl SymbolicProgram {
  // NB: you still need to add the label to the program
  pub fn create_label(&mut self, name: &'static str) -> Label {
    let id = self.next_label;
    self.next_label += 1;

    Label { id, name }
  }

  pub fn add_instruction(&mut self, inst: SymbolicInstruction) {
    self.instructions.push(inst);
  }

  pub fn parse(&self) -> Option<Vec<Instruction>> {
    let bindings = self.resolve_label_bindings()?;
    let mut instructions = Vec::new();

    for symbolic_inst in &self.instructions {
      if let &SymbolicInstruction::Instruction(kind, symbolic_op) = symbolic_inst {
        let operand = match symbolic_op {
          SymbolicOp::None => OperandType::try_from(0).expect("0 fits within u32"),
          SymbolicOp::Imm(op) => op,
          SymbolicOp::Label(label) => {
            if let Some(bound_to) = bindings.get(&label) {
              OperandType::try_from(*bound_to).ok()?
            } else {
              warn!("Unbound label: {}", label.name);
              return None;
            }
          }
        };
        instructions.push(Instruction { kind, operand });
      }
    }
    Some(instructions)
  }

  fn resolve_label_bindings(&self) -> Option<HashMap<Label, usize>> {
    let mut inst_count = 0;
    let mut bindings = HashMap::new();

    for inst in &self.instructions {
      match inst {
        &SymbolicInstruction::Label(label) => {
          if bindings.insert(label, inst_count).is_some() {
            warn!("Twice bound label: {}", label.name);
            return None;
          }
        }
        SymbolicInstruction::Instruction(_, _) => {
          inst_count += 1;
        }
      }
    }

    if bindings.values().any(|&v| v >= inst_count) {
      warn!(
        "Label bound past end of program. This can happen if you emit a label as the last symbolic instruction"
      );
      return None;
    }

    Some(bindings)
  }
}
