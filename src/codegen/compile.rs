use super::assembler::Assembler;
use super::compilation::Compilation;
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
  let mut assembler = Assembler::new();

  compile_ast(&mut assembler, ast)?;

  Ok(assembler.finalize())
}

pub fn compile_ast(assembler: &mut Assembler, ast: &Ast) -> Result<()> {
  for decl in &ast.root.declarations {
    compile_declaration(assembler, decl, ast)?;
  }
  assembler.emit(Instruction::new(InstructionKind::Return));

  Ok(())
}

fn compile_declaration(assembler: &mut Assembler, decl: &Declaration, root: &Ast) -> Result<()> {
  match decl {
    Declaration::Statement(s) => compile_statement(assembler, s, root)?,
    Declaration::Var(v) => {
      assembler.define_var(root.lexeme_arena.resolve(&v.ident), |assembler| {
        compile_expression(assembler, &v.assign, root)
      })?;
    }
  };
  Ok(())
}

fn jmp_if_not_expr(
  assembler: &mut Assembler,
  expr: &Expression,
  jump_to: Label,
  root: &Ast,
) -> Result<()> {
  compile_expression(assembler, expr, root)?;
  assembler.jmp_if_false(jump_to);

  Ok(())
}

fn compile_statement(assembler: &mut Assembler, statement: &Statement, root: &Ast) -> Result<()> {
  match statement {
    Statement::Expr(e) => {
      compile_expression(assembler, &e.expr, root)?;
      assembler.emit(Instruction::new(InstructionKind::Pop));
    }
    Statement::Print(p) => {
      compile_expression(assembler, &p.operand, root)?;
      assembler.emit(Instruction {
        kind: InstructionKind::Print,
        operand: 0,
      });
    }
    Statement::Assert(a) => {
      compile_expression(assembler, &a.operand, root)?;
      assembler.emit(Instruction::new(InstructionKind::Assert));
    }

    Statement::Block(block) => {
      compile_block(assembler, block, root)?;
    }
    Statement::If(if_statement) => {
      let after_if = assembler.create_label("after if");
      compile_if(assembler, if_statement, after_if, root)?;
      assembler.emit_symbolic(SymbolicInstruction::Label(after_if));
    }

    Statement::While(while_statement) => {
      let while_start = assembler.create_label("while start");
      let after_while = assembler.create_label("after while");
      assembler.begin_loop(after_while);
      assembler.emit_symbolic(SymbolicInstruction::Label(while_start));
      jmp_if_not_expr(assembler, &while_statement.cond, after_while, root)?;
      compile_block(assembler, &while_statement.body, root)?;
      assembler.jmp(while_start);
      assembler.emit_symbolic(SymbolicInstruction::Label(after_while));
      assembler.end_loop();
    }
    Statement::Break => assembler.loop_break()?,
  };
  Ok(())
}

fn compile_if(
  assembler: &mut Assembler,
  if_stmnt: &IfStatement,
  end_label: Label,
  root: &Ast,
) -> Result<()> {
  match if_stmnt {
    IfStatement::Trivial { cond, body } => {
      jmp_if_not_expr(assembler, cond, end_label, root)?;
      compile_block(assembler, body, root)?;
    }
    IfStatement::Fork {
      cond,
      true_case,
      false_case,
    } => {
      let after_fst_branch = assembler.create_label("after first branch");
      jmp_if_not_expr(assembler, cond, after_fst_branch, root)?;

      compile_block(assembler, true_case, root)?;
      assembler.jmp(end_label);
      assembler.emit_symbolic(SymbolicInstruction::Label(after_fst_branch));

      match false_case {
        ElseTail::Trivial(tail) => compile_block(assembler, tail, root)?,
        ElseTail::If(recurse) => compile_if(assembler, recurse, end_label, root)?,
      }
    }
  }

  Ok(())
}

fn compile_block(assembler: &mut Assembler, block: &Block, root: &Ast) -> Result<()> {
  assembler.begin_scope();

  for decl in &block.declarations {
    compile_declaration(assembler, decl, root)?;
  }

  assembler.end_scope();

  Ok(())
}

fn compile_and(
  assembler: &mut Assembler,
  lhs: &Expression,
  rhs: &Expression,
  root: &Ast,
) -> Result<()> {
  // short circuit: if we are false, return immediately. Otherwise, execute and return rhs
  let after_and = assembler.create_label("after and");
  compile_expression(assembler, lhs, root)?;

  assembler.emit_symbolic(SymbolicInstruction::Instruction(
    InstructionKind::JumpIfFalsePreserving,
    SymbolicOp::Label(after_and),
  ));

  compile_expression(assembler, rhs, root)?;
  assembler.emit_symbolic(SymbolicInstruction::Label(after_and));

  Ok(())
}

fn compile_or(
  assembler: &mut Assembler,
  lhs: &Expression,
  rhs: &Expression,
  root: &Ast,
) -> Result<()> {
  // short circuit: if we are true, return immediately. Otherwise, execute and return rhs
  let after_or = assembler.create_label("after or");
  compile_expression(assembler, lhs, root)?;

  assembler.emit_symbolic(SymbolicInstruction::Instruction(
    InstructionKind::JumpIfTruePreserving,
    SymbolicOp::Label(after_or),
  ));

  compile_expression(assembler, rhs, root)?;
  assembler.emit_symbolic(SymbolicInstruction::Label(after_or));

  Ok(())
}

fn compile_expression(assembler: &mut Assembler, expr: &Expression, root: &Ast) -> Result<()> {
  match expr {
    Expression::Bin(b) => {
      let instruction_kind = match b.op {
        BinOp::And => return compile_and(assembler, b.lhs.as_ref(), b.rhs.as_ref(), root),
        BinOp::Or => return compile_or(assembler, b.lhs.as_ref(), b.rhs.as_ref(), root),

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
      compile_expression(assembler, b.lhs.as_ref(), root)?;
      compile_expression(assembler, b.rhs.as_ref(), root)?;

      assembler.emit(Instruction {
        kind: instruction_kind,
        operand: 0,
      });
    }
    Expression::Unary(u) => {
      compile_expression(assembler, u.operand.as_ref(), root)?;
      let instruction_kind = match u.op {
        UnaryOp::Not => InstructionKind::Not,
        UnaryOp::Minus => InstructionKind::UnaryMinus,
      };
      assembler.emit(Instruction::new(instruction_kind));
    }
    Expression::Lit(literal) => {
      compile_literal(assembler, literal, root)?;
    }
    Expression::Assign(Assign { assignee, assign }) => {
      match assignee {
        LValue::Var(v) => assembler.set_variable(root.lexeme_arena.resolve(v), |assembler| {
          compile_expression(assembler, assign, root)
        })?,
      };
    }
  }

  Ok(())
}

fn compile_literal(assembler: &mut Assembler, lit: &Literal, root: &Ast) -> Result<()> {
  match lit {
    &Literal::Num(x) => assembler.constant(Constant::Float(x))?,
    Literal::String(x) => assembler.constant(Constant::String(x.clone()))?,
    &Literal::Bool(x) => assembler.constant(Constant::Bool(x))?,
    Literal::Var(v) => assembler.load_var(root.lexeme_arena.resolve(v))?,
  };

  Ok(())
}
