use super::compilation::Compilation;
use super::emitter::Emitter;
use super::error::{Error, Result};
use crate::frontend::ast::{Assign, LValue};
use crate::{
  codegen::{
    constant::Constant,
    instruction::{Instruction, InstructionKind},
  },
  frontend::ast::{Ast, BinOp, Declaration, Expression, Literal, Statement, UnaryOp},
};

pub fn compile(ast: &Ast) -> Result<Compilation> {
  let mut builder = Emitter::new();

  compile_ast(&mut builder, ast)?;

  Ok(builder.finalize())
}

pub fn compile_ast(builder: &mut Emitter, ast: &Ast) -> Result<()> {
  for decl in &ast.root.declarations {
    compile_declaration(builder, decl, ast);
  }
  builder.emit(Instruction::new(InstructionKind::Return));

  Ok(())
}

fn compile_declaration(builder: &mut Emitter, decl: &Declaration, root: &Ast) -> Result<()> {
  match decl {
    Declaration::Statement(s) => compile_statement(builder, s, root)?,
    Declaration::Var(v) => {
      compile_expression(builder, &v.assign, root)?;
      let name_idx = builder.get_or_intern_name(root.lexeme_arena.resolve(&v.ident))?;
      builder.emit(Instruction {
        kind: InstructionKind::AddGlobal,
        operand: name_idx,
      })
    }
  };
  Ok(())
}

fn compile_statement(builder: &mut Emitter, statement: &Statement, root: &Ast) -> Result<()> {
  match statement {
    Statement::Expr(e) => {
      compile_expression(builder, &e.expr, root)?;
      builder.emit(Instruction::new(InstructionKind::Pop));
    }
    Statement::Print(p) => {
      compile_expression(builder, &p.operand, root)?;
      builder.emit(Instruction {
        kind: InstructionKind::Print,
        operand: 0,
      });
    }
    Statement::Assert(a) => {
      compile_expression(builder, &a.operand, root)?;
      builder.emit(Instruction::new(InstructionKind::Assert));
    }
  };
  Ok(())
}

fn compile_expression(builder: &mut Emitter, expr: &Expression, root: &Ast) -> Result<()> {
  match expr {
    Expression::Bin(b) => {
      compile_expression(builder, b.lhs.as_ref(), root)?;
      compile_expression(builder, b.rhs.as_ref(), root)?;
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
        BinOp::Neq => InstructionKind::Neq,
      };

      builder.emit(Instruction {
        kind: instruction_kind,
        operand: 0,
      });
    }
    Expression::Unary(u) => {
      compile_expression(builder, u.operand.as_ref(), root)?;
      let instruction_kind = match u.op {
        UnaryOp::Not => InstructionKind::Not,
        UnaryOp::Minus => InstructionKind::UnaryMinus,
      };
      builder.emit(Instruction::new(instruction_kind));
    }
    Expression::Lit(literal) => {
      compile_literal(builder, literal, root)?;
    }
    // TODO: when we introduce more variable kinds (local notably),
    // include some sort of "lookup_variable" that describes uniform
    // interface of looking 'em up.
    Expression::Assign(Assign { assignee, assign }) => {
      compile_expression(builder, assign, root)?;

      let lit = match assignee {
        LValue::Var(v) => {
          let idx = builder.get_or_intern_name(root.lexeme_arena.resolve(v))?;
          builder.emit(Instruction {
            kind: InstructionKind::SetGlobal,
            operand: idx,
          });
          Literal::Var(*v)
        }
      };

      compile_literal(builder, &lit, root)?;
    }
  }

  Ok(())
}

fn compile_literal(builder: &mut Emitter, lit: &Literal, root: &Ast) -> Result<()> {
  match lit {
    &Literal::Num(x) => builder.emit_constant(Constant::Float(x))?,
    Literal::String(x) => builder.emit_constant(Constant::String(x.clone()))?,
    &Literal::Bool(x) => builder.emit_constant(Constant::Bool(x))?,
    Literal::Var(v) => {
      let idx = builder.get_or_intern_name(root.lexeme_arena.resolve(v))?;
      builder.emit(Instruction {
        kind: InstructionKind::LoadGlobal,
        operand: idx,
      })
    }
  };

  Ok(())
}
