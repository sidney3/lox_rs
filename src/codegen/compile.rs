use super::chunk_builder::ChunkBuilder;
use crate::frontend::ast::{Ast, Expression, Literal};

pub fn compile_ast(builder: &mut ChunkBuilder, ast: &Ast) {
    match ast {
        Ast::Expr(e) => compile_expression(builder, e),
    }
}

fn compile_expression(builder: &mut ChunkBuilder, expr: &Expression) {
    // NOTE: need to emit a POP afterwards
    match expr {
        Expression::Bin(b) => {
            compile_expression(builder, b.lhs.as_ref());
            compile_expression(builder, b.rhs.as_ref());
        }

        Expression::Unary(u) => {
            todo!();
        }
        Expression::Lit(literal) => {
            compile_literal(builder, literal);
        }
    }
}

fn compile_literal(builder: &mut ChunkBuilder, lit: &Literal) {
    match lit {
        Literal::Num(spur) => {
            todo!();
        }
    }
}
