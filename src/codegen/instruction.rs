use strum::FromRepr;

#[derive(FromRepr)]
#[repr(u8)]
pub enum InstructionKind {
    Return,
    Constant,
    Add,
    Sub,
    Mult,
    Divide,
    Pop,
}

pub struct Instruction {
    pub kind: InstructionKind,
    pub operand: u8,
}
