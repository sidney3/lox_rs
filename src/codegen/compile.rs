use super::chunk_builder::ChunkBuilder;
use crate::{
    codegen::{
        constant::Constant,
        instruction::{Instruction, InstructionKind},
    },
    frontend::ast::{
        Ast, BinOp, Declaration, ExprStatement, Expression, Literal, PrintStatement, Program,
        Statement,
    },
};

pub fn compile_ast(builder: &mut ChunkBuilder, ast: &Ast) {
    for decl in &ast.root.declarations {
        compile_declaration(builder, decl, ast);
    }
    builder.emit(Instruction::new(InstructionKind::Return));
}

fn compile_declaration(builder: &mut ChunkBuilder, decl: &Declaration, root: &Ast) {
    match decl {
        Declaration::Statement(s) => compile_statement(builder, s, root),
    }
}

fn compile_statement(builder: &mut ChunkBuilder, statement: &Statement, root: &Ast) {
    match statement {
        Statement::Expr(e) => {
            compile_expression(builder, &e.expr, root);
            builder.emit(Instruction::new(InstructionKind::Pop));
        }
        Statement::Print(p) => {
            compile_expression(builder, &p.operand, root);
            builder.emit(Instruction {
                kind: InstructionKind::Print,
                operand: 0,
            });
        }
    }
}

fn compile_expression(builder: &mut ChunkBuilder, expr: &Expression, root: &Ast) {
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
            compile_literal(builder, literal, root);
        }
    }
}

fn compile_literal(builder: &mut ChunkBuilder, lit: &Literal, root: &Ast) {
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
