use super::compilation::Compilation;
use super::emitter::Emitter;
use super::error::Result;
use crate::codegen::symbolic_instruction::Label;
use crate::frontend::ast::{Assign, Block, ElseTail, IfStatement, LValue};
use crate::{
  codegen::{
    constant::Constant,
    instruction::{Instruction, InstructionKind},
  },
  frontend::ast::{Ast, BinOp, Declaration, Expression, Literal, Statement, UnaryOp},
};

use super::symbolic_instruction::{SymbolicInstruction, SymbolicOp};

pub fn compile(ast: &Ast) -> Result<Compilation> {
  let mut builder = Emitter::new();

  compile_ast(&mut builder, ast)?;

  Ok(builder.finalize())
}

pub fn compile_ast(builder: &mut Emitter, ast: &Ast) -> Result<()> {
  for decl in &ast.root.declarations {
    compile_declaration(builder, decl, ast)?;
  }
  builder.emit(Instruction::new(InstructionKind::Return));

  Ok(())
}

fn compile_declaration(builder: &mut Emitter, decl: &Declaration, root: &Ast) -> Result<()> {
  match decl {
    Declaration::Statement(s) => compile_statement(builder, s, root)?,
    Declaration::Var(v) => {
      builder.emit_create_var(root.lexeme_arena.resolve(&v.ident), |builder| {
        compile_expression(builder, &v.assign, root)
      })?;
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

    Statement::Block(block) => {
      compile_block(builder, block, root)?;
    }
    Statement::If(if_statement) => {
      let after_if = builder.create_label("after if");
      compile_if(builder, if_statement, after_if, root)?;
      builder.emit_symbolic(SymbolicInstruction::Label(after_if));
    }

    Statement::While(while_statement) => {
      let while_start = builder.create_label("while start");
      let after_while = builder.create_label("after while");
      builder.begin_loop(after_while);
      builder.emit_symbolic(SymbolicInstruction::Label(while_start));
      compile_expression(builder, &while_statement.cond, root)?;
      builder.emit_symbolic(SymbolicInstruction::Instruction(
        InstructionKind::JumpIfFalse,
        SymbolicOp::Label(after_while),
      ));
      compile_block(builder, &while_statement.body, root)?;
      builder.emit_symbolic(SymbolicInstruction::Instruction(
        InstructionKind::Jmp,
        SymbolicOp::Label(while_start),
      ));
      builder.emit_symbolic(SymbolicInstruction::Label(after_while));
      builder.end_loop();
    }
  };
  Ok(())
}

fn compile_if(
  builder: &mut Emitter,
  if_stmnt: &IfStatement,
  end_label: Label,
  root: &Ast,
) -> Result<()> {
  match if_stmnt {
    IfStatement::Trivial { cond, body } => {
      compile_expression(builder, cond, root)?;
      builder.emit_symbolic(SymbolicInstruction::Instruction(
        InstructionKind::JumpIfFalse,
        SymbolicOp::Label(end_label),
      ));
      compile_block(builder, body, root)?;
    }
    IfStatement::Fork {
      cond,
      true_case,
      false_case,
    } => {
      let after_fst_branch = builder.create_label("after first branch");
      compile_expression(builder, cond, root)?;
      builder.emit_symbolic(SymbolicInstruction::Instruction(
        InstructionKind::JumpIfFalse,
        SymbolicOp::Label(after_fst_branch),
      ));

      compile_block(builder, true_case, root)?;
      builder.emit_symbolic(SymbolicInstruction::Instruction(
        InstructionKind::Jmp,
        SymbolicOp::Label(end_label),
      ));
      builder.emit_symbolic(SymbolicInstruction::Label(after_fst_branch));

      match false_case {
        ElseTail::Trivial(tail) => compile_block(builder, tail, root)?,
        ElseTail::If(recurse) => compile_if(builder, recurse, end_label, root)?,
      }
    }
  }

  Ok(())
}

fn compile_block(builder: &mut Emitter, block: &Block, root: &Ast) -> Result<()> {
  builder.begin_scope();

  for decl in &block.declarations {
    compile_declaration(builder, decl, root)?;
  }

  builder.end_scope();

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
    Expression::Assign(Assign { assignee, assign }) => {
      match assignee {
        LValue::Var(v) => builder.emit_set_variable(root.lexeme_arena.resolve(v), |builder| {
          compile_expression(builder, assign, root)
        })?,
      };
    }
  }

  Ok(())
}

fn compile_literal(builder: &mut Emitter, lit: &Literal, root: &Ast) -> Result<()> {
  match lit {
    &Literal::Num(x) => builder.emit_constant(Constant::Float(x))?,
    Literal::String(x) => builder.emit_constant(Constant::String(x.clone()))?,
    &Literal::Bool(x) => builder.emit_constant(Constant::Bool(x))?,
    Literal::Var(v) => builder.emit_load_var(root.lexeme_arena.resolve(v))?,
  };

  Ok(())
}
