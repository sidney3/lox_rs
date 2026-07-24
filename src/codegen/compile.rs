use lasso::Rodeo;
use nonempty::{NonEmpty, nonempty};

use super::error::Result;
use crate::asm::{
  Constant, FuncState, Function, Instruction, InstructionKind, Label, SymbolicInstruction,
  SymbolicOp,
};
use crate::frontend::ast::{Assign, Block, ElseTail, IfStatement, LValue};
use crate::frontend::ast::{Ast, BinOp, Declaration, Expression, Literal, Statement, UnaryOp};
use crate::frontend::token::Ident;

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
    self.ast(self.ast)?;

    assert!(self.compile_stack.len() == 1);

    let asm = self.compile_stack.into_last().assemble();

    Ok(Compilation {
      main: asm,
      symbols: self.symbols,
    })
  }

  fn ident_sym(&self, ident: Ident) -> &'a str {
    self.ast.lexeme_arena.resolve(&ident)
  }

  // Accessors to the current compiling func
  fn func_mut(&mut self) -> &mut FuncState {
    self.compile_stack.last_mut()
  }

  fn ast(&mut self, ast: &Ast) -> Result<()> {
    for decl in &ast.root.declarations {
      self.decl(decl)?;
    }
    self
      .func_mut()
      .emit(Instruction::new(InstructionKind::Return));

    Ok(())
  }

  fn decl(&mut self, decl: &Declaration) -> Result<()> {
    match decl {
      Declaration::Statement(s) => self.statement(s)?,
      Declaration::Var(v) => {
        self.expr(&v.assign)?;
        let sym = self.symbols.get_or_intern(self.ident_sym(v.ident));
        self.func_mut().define_var(sym)?;
      }
      Declaration::Fun(_) => todo!(),
    };
    Ok(())
  }

  fn statement(&mut self, statement: &Statement) -> Result<()> {
    match statement {
      Statement::Expr(e) => {
        self.expr(&e.expr)?;
        self.func_mut().emit(Instruction::new(InstructionKind::Pop));
      }
      Statement::Print(p) => {
        self.expr(&p.operand)?;
        self.func_mut().emit(Instruction {
          kind: InstructionKind::Print,
          operand: 0,
        });
      }
      Statement::Assert(a) => {
        self.expr(&a.operand)?;
        self
          .func_mut()
          .emit(Instruction::new(InstructionKind::Assert));
      }

      Statement::Block(block) => {
        self.block(block)?;
      }
      Statement::If(if_statement) => {
        let after_if = self.func_mut().create_label("after if");
        self.if_stmnt(if_statement, after_if)?;
        self.func_mut().bind_label(after_if);
      }

      Statement::While(while_statement) => {
        let while_start = self.func_mut().create_label("while start");
        let after_while = self.func_mut().create_label("after while");
        self.func_mut().begin_loop(after_while);
        self.func_mut().bind_label(while_start);

        self.expr(&while_statement.cond)?;
        self.func_mut().jmp_if_false(after_while);
        self.block(&while_statement.body)?;
        self.func_mut().jmp(while_start);
        self.func_mut().bind_label(after_while);
        self.func_mut().end_loop();
      }
      Statement::Break => self.func_mut().loop_break()?,
    };
    Ok(())
  }

  fn if_stmnt(&mut self, if_stmnt: &IfStatement, end_label: Label) -> Result<()> {
    match if_stmnt {
      IfStatement::Trivial { cond, body } => {
        self.expr(cond)?;
        self.func_mut().jmp_if_false(end_label);
        self.block(body)?;
      }
      IfStatement::Fork {
        cond,
        true_case,
        false_case,
      } => {
        let after_fst_branch = self.func_mut().create_label("after first branch");
        self.expr(cond)?;
        self.func_mut().jmp_if_false(after_fst_branch);

        self.block(true_case)?;
        self.func_mut().jmp(end_label);
        self.func_mut().bind_label(after_fst_branch);

        match false_case {
          ElseTail::Trivial(tail) => self.block(tail)?,
          ElseTail::If(recurse) => self.if_stmnt(recurse, end_label)?,
        }
      }
    }

    Ok(())
  }

  fn block(&mut self, block: &Block) -> Result<()> {
    self.func_mut().begin_scope();

    for decl in &block.declarations {
      self.decl(decl)?;
    }

    self.func_mut().end_scope();

    Ok(())
  }

  fn and(&mut self, lhs: &Expression, rhs: &Expression) -> Result<()> {
    // short circuit: if we are false, return immediately. Otherwise, execute and return rhs
    let after_and = self.func_mut().create_label("after and");
    self.expr(lhs)?;

    self
      .func_mut()
      .emit_symbolic(SymbolicInstruction::Instruction(
        InstructionKind::JumpIfFalsePreserving,
        SymbolicOp::Label(after_and),
      ));

    self.expr(rhs)?;
    self.func_mut().bind_label(after_and);

    Ok(())
  }

  fn or(&mut self, lhs: &Expression, rhs: &Expression) -> Result<()> {
    // short circuit: if we are true, return immediately. Otherwise, execute and return rhs
    let after_or = self.func_mut().create_label("after or");
    self.expr(lhs)?;

    self
      .func_mut()
      .emit_symbolic(SymbolicInstruction::Instruction(
        InstructionKind::JumpIfTruePreserving,
        SymbolicOp::Label(after_or),
      ));

    self.expr(rhs)?;
    self.func_mut().bind_label(after_or);

    Ok(())
  }

  fn expr(&mut self, expr: &Expression) -> Result<()> {
    match expr {
      Expression::Bin(b) => {
        let instruction_kind = match b.op {
          BinOp::And => return self.and(b.lhs.as_ref(), b.rhs.as_ref()),
          BinOp::Or => return self.or(b.lhs.as_ref(), b.rhs.as_ref()),

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
        self.expr(b.lhs.as_ref())?;
        self.expr(b.rhs.as_ref())?;

        self.func_mut().emit(Instruction {
          kind: instruction_kind,
          operand: 0,
        });
      }
      Expression::Unary(u) => {
        self.expr(u.operand.as_ref())?;
        let instruction_kind = match u.op {
          UnaryOp::Not => InstructionKind::Not,
          UnaryOp::Minus => InstructionKind::UnaryMinus,
        };
        self.func_mut().emit(Instruction::new(instruction_kind));
      }
      Expression::Lit(literal) => {
        self.lit(literal)?;
      }
      Expression::Assign(Assign { assignee, assign }) => {
        self.expr(assign)?;
        let assignee_sym = match assignee {
          LValue::Var(v) => self.symbols.get_or_intern(self.ident_sym(*v)),
        };
        self.func_mut().set_variable(assignee_sym)?;
      }
    }

    Ok(())
  }

  fn lit(&mut self, lit: &Literal) -> Result<()> {
    match lit {
      &Literal::Num(x) => self.func_mut().constant(Constant::Float(x))?,
      Literal::String(x) => self.func_mut().constant(Constant::String(x.clone()))?,
      &Literal::Bool(x) => self.func_mut().constant(Constant::Bool(x))?,
      Literal::Var(v) => {
        let sym = self.symbols.get_or_intern(self.ident_sym(*v));
        self.func_mut().load_var(sym)?;
      }
    };

    Ok(())
  }
}
