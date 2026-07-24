use lasso::Rodeo;
use nonempty::{NonEmpty, nonempty};

use super::error::{Error, Result};
use crate::asm::{
  Chunk, Constant, FuncState, Function, Instruction, InstructionKind, Label, SymbolicInstruction,
  SymbolicOp,
};
use crate::frontend::ast::{Assign, Block, ElseTail, IfStatement, LValue};
use crate::frontend::ast::{Ast, BinOp, Declaration, Expression, Literal, Statement, UnaryOp};

pub struct Compilation {
  pub main: Function,
  pub symbols: lasso::Rodeo,
}

pub struct Compiler<'a> {
  symbols: lasso::Rodeo,
  compile_stack: NonEmpty<FuncState>,
  ast: &'a Ast,
}

trait NonEmptyExt<T> {
  fn into_last(self) -> T;
}

impl<T> NonEmptyExt<T> for NonEmpty<T> {
  fn into_last(mut self) -> T {
    self.tail.pop().unwrap_or(self.head)
  }
}

impl<'a> Compiler<'a> {
  pub fn new(ast: &'a Ast) -> Self {
    Self {
      symbols: Rodeo::new(),
      compile_stack: nonempty![FuncState::new()],
      ast,
    }
  }

  pub fn compile(mut self) -> Result<Compilation> {
    self.compile_ast(self.ast)?;

    assert!(self.compile_stack.len() == 1);

    let asm = self.compile_stack.into_last().assemble();

    Ok(Compilation {
      main: asm,
      symbols: self.symbols,
    })
  }

  // Accessors to the current compiling func
  fn func(&self) -> &FuncState {
    self.compile_stack.last()
  }
  fn func_mut(&mut self) -> &mut FuncState {
    self.compile_stack.last_mut()
  }

  fn compile_ast(&mut self, ast: &Ast) -> Result<()> {
    for decl in &ast.root.declarations {
      self.compile_declaration(decl, ast)?;
    }
    self
      .func_mut()
      .emit(Instruction::new(InstructionKind::Return));

    Ok(())
  }

  fn compile_declaration(&mut self, decl: &Declaration, root: &Ast) -> Result<()> {
    match decl {
      Declaration::Statement(s) => self.compile_statement(s, root)?,
      Declaration::Var(v) => {
        self.compile_expression(&v.assign, root);
        let sym = self
          .symbols
          .get_or_intern(root.lexeme_arena.resolve(&v.ident));
        self.func_mut().define_var(sym)?;
      }
    };
    Ok(())
  }

  fn jmp_if_not_expr(&mut self, expr: &Expression, jump_to: Label, root: &Ast) -> Result<()> {
    self.compile_expression(expr, root)?;
    self.func_mut().jmp_if_false(jump_to);

    Ok(())
  }

  fn compile_statement(&mut self, statement: &Statement, root: &Ast) -> Result<()> {
    match statement {
      Statement::Expr(e) => {
        self.compile_expression(&e.expr, root)?;
        self.func_mut().emit(Instruction::new(InstructionKind::Pop));
      }
      Statement::Print(p) => {
        self.compile_expression(&p.operand, root)?;
        self.func_mut().emit(Instruction {
          kind: InstructionKind::Print,
          operand: 0,
        });
      }
      Statement::Assert(a) => {
        self.compile_expression(&a.operand, root)?;
        self
          .func_mut()
          .emit(Instruction::new(InstructionKind::Assert));
      }

      Statement::Block(block) => {
        self.compile_block(block, root)?;
      }
      Statement::If(if_statement) => {
        let after_if = self.func_mut().create_label("after if");
        self.compile_if(if_statement, after_if, root)?;
        self
          .func_mut()
          .emit_symbolic(SymbolicInstruction::Label(after_if));
      }

      Statement::While(while_statement) => {
        let while_start = self.func_mut().create_label("while start");
        let after_while = self.func_mut().create_label("after while");
        self.func_mut().begin_loop(after_while);
        self
          .func_mut()
          .emit_symbolic(SymbolicInstruction::Label(while_start));
        self.jmp_if_not_expr(&while_statement.cond, after_while, root)?;
        self.compile_block(&while_statement.body, root)?;
        self.func_mut().jmp(while_start);
        self
          .func_mut()
          .emit_symbolic(SymbolicInstruction::Label(after_while));
        self.func_mut().end_loop();
      }
      Statement::Break => self.func_mut().loop_break()?,
    };
    Ok(())
  }

  fn compile_if(&mut self, if_stmnt: &IfStatement, end_label: Label, root: &Ast) -> Result<()> {
    match if_stmnt {
      IfStatement::Trivial { cond, body } => {
        self.jmp_if_not_expr(cond, end_label, root)?;
        self.compile_block(body, root)?;
      }
      IfStatement::Fork {
        cond,
        true_case,
        false_case,
      } => {
        let after_fst_branch = self.func_mut().create_label("after first branch");
        self.jmp_if_not_expr(cond, after_fst_branch, root)?;

        self.compile_block(true_case, root)?;
        self.func_mut().jmp(end_label);
        self
          .func_mut()
          .emit_symbolic(SymbolicInstruction::Label(after_fst_branch));

        match false_case {
          ElseTail::Trivial(tail) => self.compile_block(tail, root)?,
          ElseTail::If(recurse) => self.compile_if(recurse, end_label, root)?,
        }
      }
    }

    Ok(())
  }

  fn compile_block(&mut self, block: &Block, root: &Ast) -> Result<()> {
    self.func_mut().begin_scope();

    for decl in &block.declarations {
      self.compile_declaration(decl, root)?;
    }

    self.func_mut().end_scope();

    Ok(())
  }

  fn compile_and(&mut self, lhs: &Expression, rhs: &Expression, root: &Ast) -> Result<()> {
    // short circuit: if we are false, return immediately. Otherwise, execute and return rhs
    let after_and = self.func_mut().create_label("after and");
    self.compile_expression(lhs, root)?;

    self
      .func_mut()
      .emit_symbolic(SymbolicInstruction::Instruction(
        InstructionKind::JumpIfFalsePreserving,
        SymbolicOp::Label(after_and),
      ));

    self.compile_expression(rhs, root)?;
    self
      .func_mut()
      .emit_symbolic(SymbolicInstruction::Label(after_and));

    Ok(())
  }

  fn compile_or(&mut self, lhs: &Expression, rhs: &Expression, root: &Ast) -> Result<()> {
    // short circuit: if we are true, return immediately. Otherwise, execute and return rhs
    let after_or = self.func_mut().create_label("after or");
    self.compile_expression(lhs, root)?;

    self
      .func_mut()
      .emit_symbolic(SymbolicInstruction::Instruction(
        InstructionKind::JumpIfTruePreserving,
        SymbolicOp::Label(after_or),
      ));

    self.compile_expression(rhs, root)?;
    self
      .func_mut()
      .emit_symbolic(SymbolicInstruction::Label(after_or));

    Ok(())
  }

  fn compile_expression(&mut self, expr: &Expression, root: &Ast) -> Result<()> {
    match expr {
      Expression::Bin(b) => {
        let instruction_kind = match b.op {
          BinOp::And => return self.compile_and(b.lhs.as_ref(), b.rhs.as_ref(), root),
          BinOp::Or => return self.compile_or(b.lhs.as_ref(), b.rhs.as_ref(), root),

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
        self.compile_expression(b.lhs.as_ref(), root)?;
        self.compile_expression(b.rhs.as_ref(), root)?;

        self.func_mut().emit(Instruction {
          kind: instruction_kind,
          operand: 0,
        });
      }
      Expression::Unary(u) => {
        self.compile_expression(u.operand.as_ref(), root)?;
        let instruction_kind = match u.op {
          UnaryOp::Not => InstructionKind::Not,
          UnaryOp::Minus => InstructionKind::UnaryMinus,
        };
        self.func_mut().emit(Instruction::new(instruction_kind));
      }
      Expression::Lit(literal) => {
        self.compile_literal(literal, root)?;
      }
      Expression::Assign(Assign { assignee, assign }) => {
        self.compile_expression(assign, root);
        let assignee_sym = match assignee {
          LValue::Var(v) => self.symbols.get_or_intern(root.lexeme_arena.resolve(v)),
        };
        self.func_mut().set_variable(assignee_sym);
      }
    }

    Ok(())
  }

  fn compile_literal(&mut self, lit: &Literal, root: &Ast) -> Result<()> {
    match lit {
      &Literal::Num(x) => self.func_mut().constant(Constant::Float(x))?,
      Literal::String(x) => self.func_mut().constant(Constant::String(x.clone()))?,
      &Literal::Bool(x) => self.func_mut().constant(Constant::Bool(x))?,
      Literal::Var(v) => {
        let sym = self.symbols.get_or_intern(root.lexeme_arena.resolve(v));
        self.func_mut().load_var(sym)?;
      }
    };

    Ok(())
  }
}
