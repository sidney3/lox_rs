use super::chunk_builder::ChunkBuilder;
use crate::{
  codegen::{
    constant::Constant,
    instruction::{Instruction, InstructionKind},
  },
  frontend::ast::{Ast, BinOp, Declaration, Expression, Literal, Statement},
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
    Statement::Assert(a) => {
      compile_expression(builder, &a.operand, root);
      builder.emit(Instruction::new(InstructionKind::Assert))
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
        BinOp::Geq => InstructionKind::Geq,
        BinOp::Leq => InstructionKind::Leq,
        BinOp::Equals => InstructionKind::Equals,
        BinOp::Less => InstructionKind::Less,
        BinOp::Greater => InstructionKind::Greater,
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
  let constant_idx = match lit {
    &Literal::Num(x) => builder.add_constant(Constant::Float(x)),
    Literal::String(x) => builder.add_constant(Constant::String(x.clone())),
    &Literal::Bool(x) => builder.add_constant(Constant::Bool(x)),
  };
  match u8::try_from(constant_idx) {
    Ok(small_idx) => builder.emit(Instruction {
      kind: InstructionKind::Constant,
      operand: small_idx,
    }),
    Err(_) => todo!("support wide indices"),
  }
}
