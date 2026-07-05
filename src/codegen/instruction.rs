use strum::FromRepr;

#[derive(FromRepr)]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum InstructionKind {
    Return,
    Constant,
    Add,
    Sub,
    Mult,
    Divide,
    Pop,
    // TODO: remove when we get functions
    // working
    Print,
}

#[derive(Copy, Clone)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub operand: u8,
}

impl Instruction {
    pub fn new(kind: InstructionKind) -> Self {
        Self { kind, operand: 0 }
    }
}
