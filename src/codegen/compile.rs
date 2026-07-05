use super::chunk_builder::ChunkBuilder;
use crate::{
    codegen::{
        constant::Constant,
        instruction::{Instruction, InstructionKind},
    },
    frontend::ast::{Ast, AstNode, BinOp, Expression, Literal},
};

pub fn compile_ast(builder: &mut ChunkBuilder, ast: &Ast) {
    match &ast.root {
        AstNode::Expr(e) => compile_expression(builder, e, ast),
    }
}

fn compile_expression(builder: &mut ChunkBuilder, expr: &Expression, root: &Ast) {
    // NOTE: need to emit a POP afterwards
    match expr {
        Expression::Bin(b) => {
            compile_expression(builder, b.lhs.as_ref(), root);
            compile_expression(builder, b.rhs.as_ref(), root);
            let instruction_kind = match b.op {
                BinOp::Times => InstructionKind::Mult,
                BinOp::Minus => InstructionKind::Sub,
                BinOp::Divide => InstructionKind::Divide,
                BinOp::Plus => InstructionKind::Add,
            };

            builder.emit(Instruction {
                kind: instruction_kind,
                operand: 0,
            })
        }
        Expression::Unary(_u) => {
            todo!();
        }
        Expression::Lit(literal) => {
            compile_literal(builder, literal);
        }
    }

    builder.emit(Instruction {
        kind: InstructionKind::Pop,
        operand: 0,
    })
}

fn compile_literal(builder: &mut ChunkBuilder, lit: &Literal) {
    match lit {
        Literal::Num(x) => {
            let idx = builder.add_constant(Constant::Float(*x));
            match u8::try_from(idx) {
                Ok(small_idx) => builder.emit(Instruction {
                    kind: InstructionKind::Constant,
                    operand: small_idx,
                }),
                Err(_) => todo!("support wide indices"),
            }
        }
    }
}
