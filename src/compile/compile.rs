use nonempty::{NonEmpty, nonempty};

use super::Compilation;
use super::error::Result;
use crate::asm::{FuncState, Instruction, InstructionKind, Label, SymbolicInstruction, SymbolicOp};
use crate::frontend::ast::{Assign, Block, ElseTail, IfStatement, LValue};
use crate::frontend::ast::{Ast, BinOp, Declaration, Expression, Literal, Statement, UnaryOp};
use crate::frontend::token::Ident;
use crate::runtime::{ObjData, Runtime, Symbol, Value};

pub struct Compiler<'a, 'vm> {
  compile_stack: NonEmpty<FuncState>,
  ast: &'a Ast,
  runtime: &'vm mut Runtime,
}

trait NonEmptyExt<T> {
  fn into_last(self) -> T;
}

impl<T> NonEmptyExt<T> for NonEmpty<T> {
  fn into_last(mut self) -> T {
    self.tail.pop().unwrap_or(self.head)
  }
}

impl<'a, 'vm> Compiler<'a, 'vm> {
  pub fn new(ast: &'a Ast, runtime: &'vm mut Runtime) -> Self {
    let main_sym = runtime.symbols.get_or_intern_static("main");
    Self {
      compile_stack: nonempty![FuncState::new(main_sym, 0)],
      ast,
      runtime,
    }
  }

  pub fn compile(mut self) -> Result<Compilation> {
    self.ast(self.ast)?;

    assert!(self.compile_stack.len() == 1);

    let main = self.compile_stack.into_last().assemble();

    Ok(Compilation {
      main: self.runtime.alloc(ObjData::Func(main)),
    })
  }

  pub fn load_ident_sym(&mut self, ident: Ident) -> Symbol {
    self.load_str_sym(self.ident_sym(ident))
  }

  pub fn load_str_sym(&mut self, s: &str) -> Symbol {
    self.runtime.symbols.get_or_intern(s)
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
        let sym = self.load_ident_sym(v.ident);
        self.func_mut().define_var(sym)?;
      }
      Declaration::Fun(f) => {
        self
          .compile_stack
          .push(FuncState::new(f.name, f.args.len()));

        // See docs/calling_convention.md
        let f_name = self.load_ident_sym(f.name);
        self.func_mut().add_local(f_name);

        for arg in &f.args {
          let arg_name = self.load_ident_sym(*arg);
          self.func_mut().add_local(arg_name);
        }

        self.block(&f.body)?;

        self.func_mut().trivial_ret()?;

        let func = self
          .compile_stack
          .pop()
          .expect("Function compilation stack too small")
          .assemble();

        let func_handle = Value::Obj(self.runtime.alloc(ObjData::Func(func)));
        self.func_mut().constant(func_handle)?;
        self.func_mut().define_var(f_name)?;
      }
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
      Statement::Return(ret) => {
        // See docs/calling_convention.md
        self.expr(&ret.expr)?;
        self.func_mut().ret();
      }
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
      Expression::Nil => {
        self.func_mut().constant(Value::Nil)?;
      }
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
          LValue::Var(v) => self.load_ident_sym(*v),
        };
        self.func_mut().set_variable(assignee_sym)?;
      }
      Expression::Call(call) => {
        // See docs/calling_convention.md
        let f_name = self.load_ident_sym(call.f);
        self.func_mut().load_var(f_name)?;
        for arg in &call.args {
          self.expr(arg)?;
        }

        self.func_mut().callq(call.args.len())?;
      }
    }

    Ok(())
  }

  fn lit(&mut self, lit: &Literal) -> Result<()> {
    match lit {
      &Literal::Num(x) => self.func_mut().constant(Value::Num(x))?,
      Literal::String(x) => {
        let string_val = Value::Obj(self.runtime.alloc(ObjData::String(x.clone())));
        self.func_mut().constant(string_val)?;
      }
      &Literal::Bool(x) => self.func_mut().constant(Value::Bool(x))?,
      Literal::Var(v) => {
        let sym = self.load_ident_sym(*v);
        self.func_mut().load_var(sym)?;
      }
    };

    Ok(())
  }
}
