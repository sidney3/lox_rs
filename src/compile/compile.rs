use super::error::Result;
use crate::asm::{
  FuncStack, FuncState, Instruction, InstructionKind, Label, SymbolicInstruction, SymbolicOp,
};
use crate::frontend::ast::{Assign, Binary, Block, Call, ElseTail, IfStatement, LValue, Unary};
use crate::frontend::ast::{Ast, BinOp, Declaration, Expression, Literal, Statement, UnaryOp};
use crate::frontend::token::Ident;
use crate::obj::{ClassDef, Function, ObjData};
use crate::runtime::{Root, Runtime, Symbol, Value};
use log::debug;

pub struct Compiler<'a, 'vm> {
  compile_stack: FuncStack,
  ast: &'a Ast,
  rt: &'vm mut Runtime,
}

impl<'a, 'vm> Compiler<'a, 'vm> {
  pub fn new(ast: &'a Ast, rt: &'vm mut Runtime) -> Self {
    let main_sym = rt.symbols.get_or_intern_static("main");
    Self {
      compile_stack: FuncStack::new(FuncState::new(rt, main_sym, 0, None)),
      ast,
      rt,
    }
  }

  pub fn compile(mut self) -> Result<Root<Function>> {
    self.ast(self.ast)?;

    let main = self
      .compile_stack
      .pop_last(self.rt)
      .expect("Finished compilation with excess remaining frames");

    Ok(main)
  }

  pub fn load_ident_sym(&mut self, ident: Ident) -> Symbol {
    self.load_str_sym(self.ident_sym(ident))
  }

  pub fn load_str_sym(&mut self, s: &str) -> Symbol {
    self.rt.symbols.get_or_intern(s)
  }

  fn ident_sym(&self, ident: Ident) -> &'a str {
    self.ast.lexeme_arena.resolve(&ident)
  }

  // Accessors to the current compiling func
  fn func_mut(&mut self) -> &mut FuncState {
    self.compile_stack.head_mut()
  }

  fn ast(&mut self, ast: &Ast) -> Result<()> {
    for decl in &ast.root.declarations {
      self.decl(decl)?;
    }
    self.func_mut().ret();

    Ok(())
  }

  fn constant(&mut self, constant: Value) -> Result<()> {
    self.compile_stack.head_mut().constant(self.rt, constant)?;

    Ok(())
  }

  fn function(
    &mut self,
    name: Symbol,
    args: &[Symbol],
    body: impl FnOnce(&mut Self) -> Result<()>,
  ) -> Result<()> {
    // See docs/calling_convention.md
    self.compile_stack.push(self.rt, name, args.len());
    self.func_mut().add_local(name);

    for arg in args {
      self.func_mut().add_local(*arg);
    }

    body(self)?;

    self.constant(Value::Nil)?;
    self.func_mut().ret();

    let func = self
      .compile_stack
      .pop(self.rt)
      .expect("Compilation stack too small");

    self
      .compile_stack
      .head_mut()
      .add_closure_to_scope(self.rt, func.as_obj(), name)?;
    func.free(self.rt);
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
      Declaration::Class(class) => {
        // We put a function called ``class.ident`` onto the
        // child class.
        let name = self.load_ident_sym(class.ident);
        let body = |this: &mut Self| -> Result<()> {
          let class_def = this.rt.alloc_typed(ClassDef::new(name)).as_root(this.rt);
          this
            .compile_stack
            .head_mut()
            .make_class_instance(this.rt, class_def.as_obj())?;
          this.func_mut().ret();
          class_def.free(this.rt);

          Ok(())
        };
        self.function(name, &[], body)?;
      }
      Declaration::Fun(f) => {
        let f_name = self.load_ident_sym(f.name);
        let args: Vec<_> = f
          .args
          .iter()
          .cloned()
          .map(|i| self.load_ident_sym(i))
          .collect();

        self.function(f_name, args.as_slice(), |this| this.block(&f.body))?;
      }
    };
    Ok(())
  }

  fn statement(&mut self, statement: &Statement) -> Result<()> {
    match statement {
      Statement::Expr(e) => {
        self.expr(&e.expr)?;
        self.func_mut().pop();
      }
      Statement::Print(p) => {
        self.expr(&p.operand)?;
        self.func_mut().print();
      }
      Statement::Assert(a) => {
        self.expr(&a.operand)?;
        self.func_mut().assert();
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

    self.func_mut().jmp_if_false_preserving(after_and);

    self.expr(rhs)?;
    self.func_mut().bind_label(after_and);

    Ok(())
  }

  fn or(&mut self, lhs: &Expression, rhs: &Expression) -> Result<()> {
    // short circuit: if we are true, return immediately. Otherwise, execute and return rhs
    let after_or = self.func_mut().create_label("after or");
    self.expr(lhs)?;
    self.func_mut().jmp_if_true_preserving(after_or);

    self.expr(rhs)?;
    self.func_mut().bind_label(after_or);

    Ok(())
  }

  fn expr(&mut self, expr: &Expression) -> Result<()> {
    match expr {
      Expression::Nil => {
        self.constant(Value::Nil)?;
      }
      Expression::Bin(Binary { op, lhs, rhs }) => {
        let instruction_kind = match op {
          BinOp::And => return self.and(lhs.as_ref(), rhs.as_ref()),
          BinOp::Or => return self.or(lhs.as_ref(), rhs.as_ref()),

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
        self.expr(lhs.as_ref())?;
        self.expr(rhs.as_ref())?;

        self.func_mut().emit(Instruction::new(instruction_kind));
      }
      Expression::Unary(Unary { op, operand }) => {
        self.expr(operand.as_ref())?;
        let instruction_kind = match op {
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
        self.expr(&call.callee)?;
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
      &Literal::Num(x) => self.constant(Value::Num(x))?,
      Literal::String(x) => {
        let string_val = Value::Obj(self.rt.alloc(ObjData::String(x.clone())));
        self.constant(string_val)?;
      }
      &Literal::Bool(x) => self.constant(Value::Bool(x))?,
      Literal::Var(v) => {
        let sym = self.load_ident_sym(*v);
        self.func_mut().load_var(sym)?;
      }
    };

    Ok(())
  }
}
