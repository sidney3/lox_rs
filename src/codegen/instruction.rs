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
}

#[derive(Copy, Clone)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub operand: u8,
}
