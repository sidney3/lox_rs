use lasso::Rodeo;

use super::error::{Error, Result};
use crate::asm::{
  Chunk, Constant, FuncState, Instruction, InstructionKind, Label, SymbolicInstruction, SymbolicOp,
};
use crate::frontend::ast::{Assign, Block, ElseTail, IfStatement, LValue};
use crate::frontend::ast::{Ast, BinOp, Declaration, Expression, Literal, Statement, UnaryOp};

pub struct Compilation {
  pub chunk: Chunk,
  pub symbols: lasso::Rodeo,
}

pub struct Compiler<'a> {
  symbols: lasso::Rodeo,
  ast: &'a Ast,
}

impl<'a> Compiler<'a> {
  pub fn new(ast: &'a Ast) -> Self {
    Self {
      symbols: Rodeo::new(),
      ast,
    }
  }

  pub fn compile(mut self) -> Result<Compilation> {
    let mut assembler = FuncState::new();

    self.compile_ast(&mut assembler, self.ast)?;

    Ok(Compilation {
      chunk: assembler.assemble(),
      symbols: self.symbols,
    })
  }

  fn compile_ast(&mut self, assembler: &mut FuncState, ast: &Ast) -> Result<()> {
    for decl in &ast.root.declarations {
      self.compile_declaration(assembler, decl, ast)?;
    }
    assembler.emit(Instruction::new(InstructionKind::Return));

    Ok(())
  }

  fn compile_declaration(
    &mut self,
    assembler: &mut FuncState,
    decl: &Declaration,
    root: &Ast,
  ) -> Result<()> {
    match decl {
      Declaration::Statement(s) => self.compile_statement(assembler, s, root)?,
      Declaration::Var(v) => {
        self.compile_expression(assembler, &v.assign, root);
        assembler.define_var(
          self
            .symbols
            .get_or_intern(root.lexeme_arena.resolve(&v.ident)),
        )?;
      }
    };
    Ok(())
  }

  fn jmp_if_not_expr(
    &mut self,
    assembler: &mut FuncState,
    expr: &Expression,
    jump_to: Label,
    root: &Ast,
  ) -> Result<()> {
    self.compile_expression(assembler, expr, root)?;
    assembler.jmp_if_false(jump_to);

    Ok(())
  }

  fn compile_statement(
    &mut self,
    assembler: &mut FuncState,
    statement: &Statement,
    root: &Ast,
  ) -> Result<()> {
    match statement {
      Statement::Expr(e) => {
        self.compile_expression(assembler, &e.expr, root)?;
        assembler.emit(Instruction::new(InstructionKind::Pop));
      }
      Statement::Print(p) => {
        self.compile_expression(assembler, &p.operand, root)?;
        assembler.emit(Instruction {
          kind: InstructionKind::Print,
          operand: 0,
        });
      }
      Statement::Assert(a) => {
        self.compile_expression(assembler, &a.operand, root)?;
        assembler.emit(Instruction::new(InstructionKind::Assert));
      }

      Statement::Block(block) => {
        self.compile_block(assembler, block, root)?;
      }
      Statement::If(if_statement) => {
        let after_if = assembler.create_label("after if");
        self.compile_if(assembler, if_statement, after_if, root)?;
        assembler.emit_symbolic(SymbolicInstruction::Label(after_if));
      }

      Statement::While(while_statement) => {
        let while_start = assembler.create_label("while start");
        let after_while = assembler.create_label("after while");
        assembler.begin_loop(after_while);
        assembler.emit_symbolic(SymbolicInstruction::Label(while_start));
        self.jmp_if_not_expr(assembler, &while_statement.cond, after_while, root)?;
        self.compile_block(assembler, &while_statement.body, root)?;
        assembler.jmp(while_start);
        assembler.emit_symbolic(SymbolicInstruction::Label(after_while));
        assembler.end_loop();
      }
      Statement::Break => assembler.loop_break()?,
    };
    Ok(())
  }

  fn compile_if(
    &mut self,
    assembler: &mut FuncState,
    if_stmnt: &IfStatement,
    end_label: Label,
    root: &Ast,
  ) -> Result<()> {
    match if_stmnt {
      IfStatement::Trivial { cond, body } => {
        self.jmp_if_not_expr(assembler, cond, end_label, root)?;
        self.compile_block(assembler, body, root)?;
      }
      IfStatement::Fork {
        cond,
        true_case,
        false_case,
      } => {
        let after_fst_branch = assembler.create_label("after first branch");
        self.jmp_if_not_expr(assembler, cond, after_fst_branch, root)?;

        self.compile_block(assembler, true_case, root)?;
        assembler.jmp(end_label);
        assembler.emit_symbolic(SymbolicInstruction::Label(after_fst_branch));

        match false_case {
          ElseTail::Trivial(tail) => self.compile_block(assembler, tail, root)?,
          ElseTail::If(recurse) => self.compile_if(assembler, recurse, end_label, root)?,
        }
      }
    }

    Ok(())
  }

  fn compile_block(&mut self, assembler: &mut FuncState, block: &Block, root: &Ast) -> Result<()> {
    assembler.begin_scope();

    for decl in &block.declarations {
      self.compile_declaration(assembler, decl, root)?;
    }

    assembler.end_scope();

    Ok(())
  }

  fn compile_and(
    &mut self,
    assembler: &mut FuncState,
    lhs: &Expression,
    rhs: &Expression,
    root: &Ast,
  ) -> Result<()> {
    // short circuit: if we are false, return immediately. Otherwise, execute and return rhs
    let after_and = assembler.create_label("after and");
    self.compile_expression(assembler, lhs, root)?;

    assembler.emit_symbolic(SymbolicInstruction::Instruction(
      InstructionKind::JumpIfFalsePreserving,
      SymbolicOp::Label(after_and),
    ));

    self.compile_expression(assembler, rhs, root)?;
    assembler.emit_symbolic(SymbolicInstruction::Label(after_and));

    Ok(())
  }

  fn compile_or(
    &mut self,
    assembler: &mut FuncState,
    lhs: &Expression,
    rhs: &Expression,
    root: &Ast,
  ) -> Result<()> {
    // short circuit: if we are true, return immediately. Otherwise, execute and return rhs
    let after_or = assembler.create_label("after or");
    self.compile_expression(assembler, lhs, root)?;

    assembler.emit_symbolic(SymbolicInstruction::Instruction(
      InstructionKind::JumpIfTruePreserving,
      SymbolicOp::Label(after_or),
    ));

    self.compile_expression(assembler, rhs, root)?;
    assembler.emit_symbolic(SymbolicInstruction::Label(after_or));

    Ok(())
  }

  fn compile_expression(
    &mut self,
    assembler: &mut FuncState,
    expr: &Expression,
    root: &Ast,
  ) -> Result<()> {
    match expr {
      Expression::Bin(b) => {
        let instruction_kind = match b.op {
          BinOp::And => return self.compile_and(assembler, b.lhs.as_ref(), b.rhs.as_ref(), root),
          BinOp::Or => return self.compile_or(assembler, b.lhs.as_ref(), b.rhs.as_ref(), root),

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
        self.compile_expression(assembler, b.lhs.as_ref(), root)?;
        self.compile_expression(assembler, b.rhs.as_ref(), root)?;

        assembler.emit(Instruction {
          kind: instruction_kind,
          operand: 0,
        });
      }
      Expression::Unary(u) => {
        self.compile_expression(assembler, u.operand.as_ref(), root)?;
        let instruction_kind = match u.op {
          UnaryOp::Not => InstructionKind::Not,
          UnaryOp::Minus => InstructionKind::UnaryMinus,
        };
        assembler.emit(Instruction::new(instruction_kind));
      }
      Expression::Lit(literal) => {
        self.compile_literal(assembler, literal, root)?;
      }
      Expression::Assign(Assign { assignee, assign }) => {
        self.compile_expression(assembler, assign, root);
        match assignee {
          LValue::Var(v) => {
            assembler.set_variable(self.symbols.get_or_intern(root.lexeme_arena.resolve(v)))?
          }
        };
      }
    }

    Ok(())
  }

  fn compile_literal(
    &mut self,
    assembler: &mut FuncState,
    lit: &Literal,
    root: &Ast,
  ) -> Result<()> {
    match lit {
      &Literal::Num(x) => assembler.constant(Constant::Float(x))?,
      Literal::String(x) => assembler.constant(Constant::String(x.clone()))?,
      &Literal::Bool(x) => assembler.constant(Constant::Bool(x))?,
      Literal::Var(v) => {
        assembler.load_var(self.symbols.get_or_intern(root.lexeme_arena.resolve(v)))?
      }
    };

    Ok(())
  }
}
