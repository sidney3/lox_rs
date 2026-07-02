use strum::FromRepr;

#[derive(FromRepr)]
#[repr(u8)]
pub enum InstructionKind {
    Return,
    Constant,
    Add,
    Sub,
}

pub struct Instruction {
    kind: InstructionKind,
    operand: u8,
}
