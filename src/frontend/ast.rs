use std::vec;

use lox_derive::Ordinal;
use nonempty::{NonEmpty, nonempty};
use strum::Display;

use super::error::Result;
use super::token::LoxTokenKind;
use crate::frontend::token::Ident;
use crate::lexer::Tokens;
use crate::parse::{Grammar, Node, Parent, Parser, Production, Rule, Symbol, Tree};

#[derive(Ordinal, Eq, PartialEq, Hash, Display, Debug, PartialOrd)]
pub enum LoxRule {
  Paren,
  Term,
  Product,
  Compare,
  Expr,
  Literal,
  Unary,
  Program,
  Print,
  Assert,
  Declaration,
  FuncDecl,
  FuncArgs,
  NonemptyFuncArgs,
  VarDecl,
  Statement,
  ExprStatement,
  BlockStatement,
  IfStatement,
  ElseTail,
  WhileStatement,
  Unit,
  Call,
  CallArgs,
  NonemptyCallArgs,
  Return,

  LValue,
  Assign,
}

impl Rule for LoxRule {
  type TokenType = LoxTokenKind;
}

#[derive(Debug)]
pub enum BinOp {
  Times,
  Divide,
  Plus,
  Minus,

  Equals,
  Less,
  Leq,
  Greater,
  Geq,
  Neq,

  And,
  Or,
}

impl BinOp {
  fn from_token(token: LoxTokenKind) -> Option<Self> {
    match token {
      LoxTokenKind::Star => Some(Self::Times),
      LoxTokenKind::Slash => Some(Self::Divide),
      LoxTokenKind::Plus => Some(Self::Plus),
      LoxTokenKind::Minus => Some(Self::Minus),
      LoxTokenKind::EqualEqual => Some(Self::Equals),
      LoxTokenKind::Less => Some(Self::Less),
      LoxTokenKind::LessEqual => Some(Self::Leq),
      LoxTokenKind::Greater => Some(Self::Greater),
      LoxTokenKind::GreaterEqual => Some(Self::Geq),
      LoxTokenKind::BangEqual => Some(Self::Neq),
      LoxTokenKind::And => Some(Self::And),
      LoxTokenKind::Or => Some(Self::Or),
      _ => None,
    }
  }
}

pub struct Ast {
  pub lexeme_arena: lasso::Rodeo,
  pub root: Program,
}

#[derive(Debug)]
pub enum UnaryOp {
  Minus,
  Not,
}

impl UnaryOp {
  fn from_token(token: LoxTokenKind) -> Option<Self> {
    match token {
      LoxTokenKind::Minus => Some(Self::Minus),
      LoxTokenKind::Bang => Some(Self::Not),
      _ => None,
    }
  }
}

#[derive(Debug)]
pub struct Binary {
  pub lhs: Box<Expression>,
  pub op: BinOp,
  pub rhs: Box<Expression>,
}

#[derive(Debug)]
pub struct Unary {
  pub operand: Box<Expression>,
  pub op: UnaryOp,
}

#[derive(Debug)]
pub enum Literal {
  Num(f64),
  String(String),
  Bool(bool),
  Var(lasso::Spur),
}

#[derive(Debug)]
pub struct Call {
  pub f: Ident,
  pub args: Vec<Expression>,
}

#[derive(Debug)]
pub enum Expression {
  Bin(Binary),
  Unary(Unary),
  Lit(Literal),
  Assign(Assign),
  Call(Call),
  Nil,
}

#[derive(Debug)]
pub struct ExprStatement {
  pub expr: Expression,
}

#[derive(Debug)]
pub struct PrintStatement {
  pub operand: Expression,
}

#[derive(Debug)]
pub struct AssertStatement {
  pub operand: Expression,
}

#[derive(Debug)]
pub struct WhileStatement {
  pub cond: Expression,
  pub body: Box<Block>,
}

#[derive(Debug)]
pub enum IfStatement {
  Trivial {
    cond: Expression,
    body: Box<Block>,
  },
  Fork {
    cond: Expression,
    true_case: Box<Block>,
    false_case: ElseTail,
  },
}

#[derive(Debug)]
pub enum ElseTail {
  Trivial(Box<Block>),
  If(Box<IfStatement>),
}

#[derive(Debug)]
pub struct Block {
  pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub struct Return {
  pub expr: Expression,
}

#[derive(Debug)]
pub enum Statement {
  Expr(ExprStatement),
  Print(PrintStatement),
  Assert(AssertStatement),
  Block(Block),
  If(IfStatement),
  While(WhileStatement),
  Return(Return),
  Break,
}

#[derive(Debug)]
pub struct FuncDecl {
  pub name: Ident,
  pub body: Block,
  pub args: Vec<Ident>,
}

#[derive(Debug)]
pub struct VarDeclaration {
  pub ident: lasso::Spur,
  pub assign: Expression,
}

#[derive(Debug)]
pub enum Declaration {
  Statement(Statement),
  Var(VarDeclaration),
  Fun(FuncDecl),
}

#[derive(Debug)]
pub enum LValue {
  Var(lasso::Spur),
}

#[derive(Debug)]
pub struct Assign {
  pub assignee: LValue,
  pub assign: Box<Expression>,
}

#[derive(Debug)]
pub struct Program {
  pub declarations: NonEmpty<Declaration>,
}

type P = Production<LoxRule>;

fn lox_grammar() -> Grammar<LoxRule> {
  Grammar::new(
    LoxRule::Program, // goal rule
    vec![
      // Program := Declaration | Program Declaration;
      P {
        rule: LoxRule::Program,
        definition: vec![Symbol::Rule(LoxRule::Declaration)],
      },
      P {
        rule: LoxRule::Program,
        definition: vec![
          Symbol::Rule(LoxRule::Program),
          Symbol::Rule(LoxRule::Declaration),
        ],
      },
      // Declaration := Statement | VarDeclaration | FuncDec;
      P {
        rule: LoxRule::Declaration,
        definition: vec![Symbol::Rule(LoxRule::Statement)],
      },
      P {
        rule: LoxRule::Declaration,
        definition: vec![Symbol::Rule(LoxRule::FuncDecl)],
      },
      P {
        rule: LoxRule::Declaration,
        definition: vec![Symbol::Rule(LoxRule::VarDecl)],
      },
      // VarDecl := 'var' 'ident' '=' 'expr' ;
      P {
        rule: LoxRule::VarDecl,
        definition: vec![
          Symbol::Token(LoxTokenKind::Var),
          Symbol::Token(LoxTokenKind::Ident),
          Symbol::Token(LoxTokenKind::Equal),
          Symbol::Rule(LoxRule::Expr),
          Symbol::Token(LoxTokenKind::Semicolon),
        ],
      },
      // FuncDecl := 'fun' 'Ident' '(' func_args ')' '{' block '}'
      P {
        rule: LoxRule::FuncDecl,
        definition: vec![
          Symbol::Token(LoxTokenKind::Fun),
          Symbol::Token(LoxTokenKind::Ident),
          Symbol::Token(LoxTokenKind::LParen),
          Symbol::Rule(LoxRule::FuncArgs),
          Symbol::Token(LoxTokenKind::RParen),
          Symbol::Token(LoxTokenKind::LBracket),
          Symbol::Rule(LoxRule::BlockStatement),
          Symbol::Token(LoxTokenKind::RBracket),
        ],
      },
      // FuncArgs := ε | NonemptyFuncArgs
      P {
        rule: LoxRule::FuncArgs,
        definition: vec![],
      },
      // FuncArgs := ε | NonemptyFuncArgs
      P {
        rule: LoxRule::FuncArgs,
        definition: vec![Symbol::Rule(LoxRule::NonemptyFuncArgs)],
      },
      // NonemptyFuncArgs := 'ident' | NonemptyFuncArgs ',' 'ident'
      P {
        rule: LoxRule::NonemptyFuncArgs,
        definition: vec![Symbol::Token(LoxTokenKind::Ident)],
      },
      P {
        rule: LoxRule::NonemptyFuncArgs,
        definition: vec![
          Symbol::Rule(LoxRule::NonemptyFuncArgs),
          Symbol::Token(LoxTokenKind::Comma),
          Symbol::Token(LoxTokenKind::Ident),
        ],
      },
      // Statement :=
      // AssertStatement
      // | PrintStatement
      // | ExpressionStatemt
      // | '{' BlockStatement '}';
      // | IfStatement
      // | WhileStatement
      // | Return
      // | Break
      P {
        rule: LoxRule::Statement,
        definition: vec![Symbol::Rule(LoxRule::Print)],
      },
      P {
        rule: LoxRule::Statement,
        definition: vec![Symbol::Rule(LoxRule::Assert)],
      },
      P {
        rule: LoxRule::Statement,
        definition: vec![Symbol::Rule(LoxRule::ExprStatement)],
      },
      P {
        rule: LoxRule::Statement,
        definition: vec![
          Symbol::Token(LoxTokenKind::LBracket),
          Symbol::Rule(LoxRule::BlockStatement),
          Symbol::Token(LoxTokenKind::RBracket),
        ],
      },
      P {
        rule: LoxRule::Statement,
        definition: vec![Symbol::Rule(LoxRule::IfStatement)],
      },
      P {
        rule: LoxRule::Statement,
        definition: vec![Symbol::Rule(LoxRule::WhileStatement)],
      },
      P {
        rule: LoxRule::Statement,
        definition: vec![Symbol::Rule(LoxRule::Return)],
      },
      P {
        rule: LoxRule::Statement,
        definition: vec![
          Symbol::Token(LoxTokenKind::Break),
          Symbol::Token(LoxTokenKind::Semicolon),
        ],
      },
      // BlockStatement := ε | BlockStatement Declaration
      P {
        rule: LoxRule::BlockStatement,
        definition: vec![],
      },
      P {
        rule: LoxRule::BlockStatement,
        definition: vec![
          Symbol::Rule(LoxRule::BlockStatement),
          Symbol::Rule(LoxRule::Declaration),
        ],
      },
      // PrintStatement := 'print' Expr ';'
      P {
        rule: LoxRule::Print,
        definition: vec![
          Symbol::Token(LoxTokenKind::Print),
          Symbol::Rule(LoxRule::Expr),
          Symbol::Token(LoxTokenKind::Semicolon),
        ],
      },
      // AssertStatement := 'assert' Expr ';'
      P {
        rule: LoxRule::Assert,
        definition: vec![
          Symbol::Token(LoxTokenKind::Assert),
          Symbol::Rule(LoxRule::Expr),
          Symbol::Token(LoxTokenKind::Semicolon),
        ],
      },
      // ExprStatement := Expr ';'
      P {
        rule: LoxRule::ExprStatement,
        definition: vec![
          Symbol::Rule(LoxRule::Expr),
          Symbol::Token(LoxTokenKind::Semicolon),
        ],
      },
      // IfStatement := 'if' expr '{' body '}' | 'if' expr '{' body '}' 'else' else_tail
      P {
        rule: LoxRule::IfStatement,
        definition: vec![
          Symbol::Token(LoxTokenKind::If),
          Symbol::Rule(LoxRule::Expr),
          Symbol::Token(LoxTokenKind::LBracket),
          Symbol::Rule(LoxRule::BlockStatement),
          Symbol::Token(LoxTokenKind::RBracket),
        ],
      },
      P {
        rule: LoxRule::IfStatement,
        definition: vec![
          Symbol::Token(LoxTokenKind::If),
          Symbol::Rule(LoxRule::Expr),
          Symbol::Token(LoxTokenKind::LBracket),
          Symbol::Rule(LoxRule::BlockStatement),
          Symbol::Token(LoxTokenKind::RBracket),
          Symbol::Token(LoxTokenKind::Else),
          Symbol::Rule(LoxRule::ElseTail),
        ],
      },
      // ElseTail := '{' body '}' | IfStatement
      P {
        rule: LoxRule::ElseTail,
        definition: vec![
          Symbol::Token(LoxTokenKind::LBracket),
          Symbol::Rule(LoxRule::BlockStatement),
          Symbol::Token(LoxTokenKind::RBracket),
        ],
      },
      P {
        rule: LoxRule::ElseTail,
        definition: vec![Symbol::Rule(LoxRule::IfStatement)],
      },
      // WhileStatement := 'while' cond '{' body '}'
      P {
        rule: LoxRule::WhileStatement,
        definition: vec![
          Symbol::Token(LoxTokenKind::While),
          Symbol::Rule(LoxRule::Expr),
          Symbol::Token(LoxTokenKind::LBracket),
          Symbol::Rule(LoxRule::BlockStatement),
          Symbol::Token(LoxTokenKind::RBracket),
        ],
      },
      // ReturnStatement := 'return' ';' | 'return' expr ';' ;
      P {
        rule: LoxRule::Return,
        definition: vec![
          Symbol::Token(LoxTokenKind::Return),
          Symbol::Token(LoxTokenKind::Semicolon),
        ],
      },
      P {
        rule: LoxRule::Return,
        definition: vec![
          Symbol::Token(LoxTokenKind::Return),
          Symbol::Rule(LoxRule::Expr),
          Symbol::Token(LoxTokenKind::Semicolon),
        ],
      },
      // ------------------
      // EXPRESSION GRAMMAR
      // ------------------
      // Expr := Assign
      P {
        rule: LoxRule::Expr,
        definition: vec![Symbol::Rule(LoxRule::Assign)],
      },
      // Assign :=
      //       | lvalue '=' Expr
      //       | Compare
      //
      P {
        rule: LoxRule::Assign,
        definition: vec![
          Symbol::Rule(LoxRule::LValue),
          Symbol::Token(LoxTokenKind::Equal),
          Symbol::Rule(LoxRule::Expr),
        ],
      },
      P {
        rule: LoxRule::Assign,
        definition: vec![Symbol::Rule(LoxRule::Compare)],
      },
      P {
        rule: LoxRule::LValue,
        definition: vec![Symbol::Token(LoxTokenKind::Ident)],
      },
      // Compare :=
      //       | Term '==' Compare
      //       | Term '>=' Compare
      //       | Term '>' Compare
      //       | Term '<' Compare
      //       | Term '<=' Compare
      //       | Term '!=' Compare
      //       | Term '&&' Compare
      //       | Term '||' Compare
      //       | Term
      //
      P {
        rule: LoxRule::Compare,
        definition: vec![
          Symbol::Rule(LoxRule::Term),
          Symbol::Token(LoxTokenKind::EqualEqual),
          Symbol::Rule(LoxRule::Compare),
        ],
      },
      P {
        rule: LoxRule::Compare,
        definition: vec![
          Symbol::Rule(LoxRule::Term),
          Symbol::Token(LoxTokenKind::LessEqual),
          Symbol::Rule(LoxRule::Compare),
        ],
      },
      P {
        rule: LoxRule::Compare,
        definition: vec![
          Symbol::Rule(LoxRule::Term),
          Symbol::Token(LoxTokenKind::Less),
          Symbol::Rule(LoxRule::Compare),
        ],
      },
      P {
        rule: LoxRule::Compare,
        definition: vec![
          Symbol::Rule(LoxRule::Term),
          Symbol::Token(LoxTokenKind::GreaterEqual),
          Symbol::Rule(LoxRule::Compare),
        ],
      },
      P {
        rule: LoxRule::Compare,
        definition: vec![
          Symbol::Rule(LoxRule::Term),
          Symbol::Token(LoxTokenKind::Greater),
          Symbol::Rule(LoxRule::Compare),
        ],
      },
      P {
        rule: LoxRule::Compare,
        definition: vec![
          Symbol::Rule(LoxRule::Term),
          Symbol::Token(LoxTokenKind::BangEqual),
          Symbol::Rule(LoxRule::Compare),
        ],
      },
      P {
        rule: LoxRule::Compare,
        definition: vec![
          Symbol::Rule(LoxRule::Term),
          Symbol::Token(LoxTokenKind::And),
          Symbol::Rule(LoxRule::Compare),
        ],
      },
      P {
        rule: LoxRule::Compare,
        definition: vec![
          Symbol::Rule(LoxRule::Term),
          Symbol::Token(LoxTokenKind::Or),
          Symbol::Rule(LoxRule::Compare),
        ],
      },
      P {
        rule: LoxRule::Compare,
        definition: vec![Symbol::Rule(LoxRule::Term)],
      },
      //
      // Term := Product '+' Term
      //       | Product '-' Term
      //       | Product
      P {
        rule: LoxRule::Term,
        definition: vec![
          Symbol::Rule(LoxRule::Term),
          Symbol::Token(LoxTokenKind::Plus),
          Symbol::Rule(LoxRule::Product),
        ],
      },
      P {
        rule: LoxRule::Term,
        definition: vec![
          Symbol::Rule(LoxRule::Term),
          Symbol::Token(LoxTokenKind::Minus),
          Symbol::Rule(LoxRule::Product),
        ],
      },
      P {
        rule: LoxRule::Term,
        definition: vec![Symbol::Rule(LoxRule::Product)],
      },
      // Product := Unary | Product '*' Unary | Product '/' Unary
      P {
        rule: LoxRule::Product,
        definition: vec![Symbol::Rule(LoxRule::Unary)],
      },
      P {
        rule: LoxRule::Product,
        definition: vec![
          Symbol::Rule(LoxRule::Product),
          Symbol::Token(LoxTokenKind::Star),
          Symbol::Rule(LoxRule::Unary),
        ],
      },
      P {
        rule: LoxRule::Product,
        definition: vec![
          Symbol::Rule(LoxRule::Product),
          Symbol::Token(LoxTokenKind::Slash),
          Symbol::Rule(LoxRule::Unary),
        ],
      },
      // Unary :=
      //       '-' Unary
      //      | '!' Unary
      //      | Paren
      //
      P {
        rule: LoxRule::Unary,
        definition: vec![
          Symbol::Token(LoxTokenKind::Bang),
          Symbol::Rule(LoxRule::Unary),
        ],
      },
      P {
        rule: LoxRule::Unary,
        definition: vec![
          Symbol::Token(LoxTokenKind::Minus),
          Symbol::Rule(LoxRule::Unary),
        ],
      },
      P {
        rule: LoxRule::Unary,
        definition: vec![Symbol::Rule(LoxRule::Paren)],
      },
      // Paren := Unit | '(' Expr ')'
      P {
        rule: LoxRule::Paren,
        definition: vec![Symbol::Rule(LoxRule::Unit)],
      },
      P {
        rule: LoxRule::Paren,
        definition: vec![
          Symbol::Token(LoxTokenKind::LParen),
          Symbol::Rule(LoxRule::Expr),
          Symbol::Token(LoxTokenKind::RParen),
        ],
      },
      // Unit := Literal | Call
      P {
        rule: LoxRule::Unit,
        definition: vec![Symbol::Rule(LoxRule::Literal)],
      },
      P {
        rule: LoxRule::Unit,
        definition: vec![Symbol::Rule(LoxRule::Call)],
      },
      // Call := ident '( func_args ')'
      P {
        rule: LoxRule::Call,
        definition: vec![
          Symbol::Token(LoxTokenKind::Ident),
          Symbol::Token(LoxTokenKind::LParen),
          Symbol::Rule(LoxRule::CallArgs),
          Symbol::Token(LoxTokenKind::RParen),
        ],
      },
      // CallArgs := ε | NonemptyCallArgs
      P {
        rule: LoxRule::CallArgs,
        definition: Vec::new(),
      },
      P {
        rule: LoxRule::CallArgs,
        definition: vec![Symbol::Rule(LoxRule::NonemptyCallArgs)],
      },
      // ArgList := Expr | ArgList ',' Expr;
      P {
        rule: LoxRule::NonemptyCallArgs,
        definition: vec![Symbol::Rule(LoxRule::Expr)],
      },
      P {
        rule: LoxRule::NonemptyCallArgs,
        definition: vec![
          Symbol::Rule(LoxRule::NonemptyCallArgs),
          Symbol::Token(LoxTokenKind::Comma),
          Symbol::Rule(LoxRule::Expr),
        ],
      },
      // Literal := 'Num' | 'Str' | 'True' | 'False' | 'Ident' | 'Nil'
      P {
        rule: LoxRule::Literal,
        definition: vec![Symbol::Token(LoxTokenKind::Number)],
      },
      P {
        rule: LoxRule::Literal,
        definition: vec![Symbol::Token(LoxTokenKind::String)],
      },
      P {
        rule: LoxRule::Literal,
        definition: vec![Symbol::Token(LoxTokenKind::True)],
      },
      P {
        rule: LoxRule::Literal,
        definition: vec![Symbol::Token(LoxTokenKind::False)],
      },
      P {
        rule: LoxRule::Literal,
        definition: vec![Symbol::Token(LoxTokenKind::Ident)],
      },
      P {
        rule: LoxRule::Literal,
        definition: vec![Symbol::Token(LoxTokenKind::Nil)],
      },
    ],
  )
}

pub struct LoxParser(Parser<LoxRule>);

impl LoxParser {
  pub fn new() -> Self {
    Self(Parser::<LoxRule>::new(lox_grammar()))
  }

  pub fn parse(&self, tokens: Tokens<LoxTokenKind>) -> Result<Ast> {
    let cst = self.0.parse(tokens)?;

    Ok(Ast::from_cst(cst))
  }
}

impl ExprStatement {
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::ExprStatement,
        children,
      }) => match children.as_slice() {
        [expr, Node::Leaf(semicolon)] if semicolon.token_type == LoxTokenKind::Semicolon => {
          ExprStatement {
            expr: Expression::from_cst(ast, expr),
          }
        }
        _ => panic!("unreachable: {:?}", node),
      },
      _ => panic!("unreachable: {:?}", node),
    }
  }
}

impl PrintStatement {
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::Print,
        children,
      }) => match children.as_slice() {
        [Node::Leaf(print), expr, Node::Leaf(semicolon)]
          if print.token_type == LoxTokenKind::Print
            && semicolon.token_type == LoxTokenKind::Semicolon =>
        {
          PrintStatement {
            operand: Expression::from_cst(ast, expr),
          }
        }
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
    }
  }
}

impl AssertStatement {
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::Assert,
        children,
      }) => match children.as_slice() {
        [Node::Leaf(print), expr, Node::Leaf(semicolon)]
          if print.token_type == LoxTokenKind::Assert
            && semicolon.token_type == LoxTokenKind::Semicolon =>
        {
          AssertStatement {
            operand: Expression::from_cst(ast, expr),
          }
        }
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable: {:?}", node),
    }
  }
}

impl WhileStatement {
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::WhileStatement,
        children,
      }) => match children.as_slice() {
        [_, cond, _, body, _] => WhileStatement {
          cond: Expression::from_cst(ast, cond),
          body: Box::new(Block::from_cst(ast, body)),
        },
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable: {:?}", node),
    }
  }
}

impl IfStatement {
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::IfStatement,
        children,
      }) => match children.as_slice() {
        [_, cond, _, body, _] => IfStatement::Trivial {
          cond: Expression::from_cst(ast, cond),
          body: Box::new(Block::from_cst(ast, body)),
        },
        [_, cond, _, true_case, _, _, else_tail] => IfStatement::Fork {
          cond: Expression::from_cst(ast, cond),
          true_case: Box::new(Block::from_cst(ast, true_case)),
          false_case: ElseTail::from_cst(ast, else_tail),
        },
        _ => panic!("unreachable: {:?}", node),
      },
      _ => panic!("unreachable: {:?}", node),
    }
  }
}

impl ElseTail {
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::ElseTail,
        children,
      }) => match children.as_slice() {
        [if_stmnt] => ElseTail::If(Box::new(IfStatement::from_cst(ast, if_stmnt))),
        [_, block, _] => ElseTail::Trivial(Box::new(Block::from_cst(ast, block))),
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
    }
  }
}

impl Return {
  pub fn new(expr: Expression) -> Self {
    Self { expr }
  }
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::Return,
        children,
      }) => match children.as_slice() {
        [Node::Leaf(_return), Node::Leaf(_semicolon)]
          if _return.token_type == LoxTokenKind::Return =>
        {
          Return::new(Expression::Nil)
        }
        [Node::Leaf(_return), expr, Node::Leaf(_semicolon)]
          if _return.token_type == LoxTokenKind::Return =>
        {
          Return::new(Expression::from_cst(ast, expr))
        }
        _ => panic!("unreachable: {:?}", node),
      },
      _ => panic!("unreachable"),
    }
  }
}

impl Statement {
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::Statement,
        children,
      }) => match children.as_slice() {
        [
          Node::Parent(Parent {
            rule: LoxRule::Print,
            children: _,
          }),
        ] => Statement::Print(PrintStatement::from_cst(ast, &children[0])),
        [
          Node::Parent(Parent {
            rule: LoxRule::Assert,
            children: _,
          }),
        ] => Statement::Assert(AssertStatement::from_cst(ast, &children[0])),
        [
          Node::Parent(Parent {
            rule: LoxRule::IfStatement,
            children: _,
          }),
        ] => Statement::If(IfStatement::from_cst(ast, &children[0])),
        [
          Node::Parent(Parent {
            rule: LoxRule::WhileStatement,
            children: _,
          }),
        ] => Statement::While(WhileStatement::from_cst(ast, &children[0])),
        [
          Node::Parent(Parent {
            rule: LoxRule::ExprStatement,
            children: _,
          }),
        ] => Statement::Expr(ExprStatement::from_cst(ast, &children[0])),
        [
          Node::Parent(Parent {
            rule: LoxRule::Return,
            children: _,
          }),
        ] => Statement::Return(Return::from_cst(ast, &children[0])),
        [Node::Leaf(l), block, Node::Leaf(r)]
          if l.token_type == LoxTokenKind::LBracket && r.token_type == LoxTokenKind::RBracket =>
        {
          Statement::Block(Block::from_cst(ast, block))
        }
        [Node::Leaf(l), Node::Leaf(_)] if l.token_type == LoxTokenKind::Break => Statement::Break,
        _ => panic!("unreachable: {:?}", node),
      },
      _ => panic!("unreachable: {:?}", node),
    }
  }
}

impl VarDeclaration {
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::VarDecl,
        children,
      }) => match children.as_slice() {
        [
          Node::Leaf(var),
          Node::Leaf(ident),
          Node::Leaf(eq),
          assign,
          Node::Leaf(semicolon),
        ] if var.token_type == LoxTokenKind::Var
          && ident.token_type == LoxTokenKind::Ident
          && eq.token_type == LoxTokenKind::Equal
          && semicolon.token_type == LoxTokenKind::Semicolon =>
        {
          VarDeclaration {
            ident: ident.lexeme,
            assign: Expression::from_cst(ast, assign),
          }
        }
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
    }
  }
}

impl FuncDecl {
  fn parse_nonempty_args(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Vec<Ident> {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::NonemptyFuncArgs,
        children,
      }) => match children.as_slice() {
        [Node::Leaf(ident)] => vec![ident.lexeme],
        [elts, Node::Leaf(_comma), Node::Leaf(ident)] => {
          let mut parsed_elts = Self::parse_nonempty_args(ast, elts);
          parsed_elts.push(ident.lexeme);
          parsed_elts
        }
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
    }
  }
  fn parse_args(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Vec<Ident> {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::FuncArgs,
        children,
      }) => match children.as_slice() {
        [] => Vec::new(),
        [arg_list] => Self::parse_nonempty_args(ast, arg_list),
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
    }
  }
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::FuncDecl,
        children,
      }) => match children.as_slice() {
        [
          Node::Leaf(_fun),
          Node::Leaf(name),
          _lparen,
          args,
          Node::Leaf(_rparen),
          _lbrace,
          body,
          Node::Leaf(_rbrace),
        ] => FuncDecl {
          name: name.lexeme,
          body: Block::from_cst(ast, body),
          args: Self::parse_args(ast, args),
        },
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
    }
  }
}

impl Declaration {
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::Declaration,
        children,
      }) => match children.as_slice() {
        [
          Node::Parent(Parent {
            rule: LoxRule::VarDecl,
            children: _,
          }),
        ] => Declaration::Var(VarDeclaration::from_cst(ast, &children[0])),
        [
          Node::Parent(Parent {
            rule: LoxRule::FuncDecl,
            children: _,
          }),
        ] => Declaration::Fun(FuncDecl::from_cst(ast, &children[0])),
        [
          Node::Parent(Parent {
            rule: LoxRule::Statement,
            children: _,
          }),
        ] => Declaration::Statement(Statement::from_cst(ast, &children[0])),
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable: {:?}", node),
    }
  }
}

impl Program {
  pub fn new(last: Declaration) -> Self {
    Self {
      declarations: nonempty![last],
    }
  }

  pub fn push(mut self, next: Declaration) -> Self {
    self.declarations.push(next);
    self
  }
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::Program,
        children,
      }) => match children.as_slice() {
        [program_node, decl_node] => {
          Program::from_cst(ast, program_node).push(Declaration::from_cst(ast, decl_node))
        }
        [decl_node] => Program::new(Declaration::from_cst(ast, decl_node)),
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
    }
  }
}

impl LValue {
  pub fn from_cst(_: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::LValue,
        children,
      }) => match children.as_slice() {
        [Node::Leaf(token)] if token.token_type == LoxTokenKind::Ident => LValue::Var(token.lexeme),
        _ => panic!("unreachable: {:?}", node),
      },
      Node::Leaf(token) if token.token_type == LoxTokenKind::Ident => LValue::Var(token.lexeme),
      _ => panic!("unreachable: {:?}", node),
    }
  }
}

impl Block {
  pub fn new() -> Self {
    Self {
      declarations: Vec::new(),
    }
  }

  pub fn push(mut self, next: Declaration) -> Self {
    self.declarations.push(next);
    self
  }
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::BlockStatement,
        children,
      }) => match children.as_slice() {
        [] => Block::new(),
        [block, statement] => {
          Block::from_cst(ast, block).push(Declaration::from_cst(ast, statement))
        }
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
    }
  }
}

impl Call {
  fn parse_nonempty_args(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Vec<Expression> {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::NonemptyCallArgs,
        children,
      }) => match children.as_slice() {
        [expr] => vec![Expression::from_cst(ast, expr)],
        [elts, Node::Leaf(_comma), expr] => {
          let mut parsed_elts = Self::parse_nonempty_args(ast, elts);
          parsed_elts.push(Expression::from_cst(ast, expr));
          parsed_elts
        }
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
    }
  }
  fn parse_args(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Vec<Expression> {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::CallArgs,
        children,
      }) => match children.as_slice() {
        [] => Vec::new(),
        [arg_list] => Self::parse_nonempty_args(ast, arg_list),
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
    }
  }
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::Call,
        children,
      }) => match children.as_slice() {
        [Node::Leaf(f), _lparen, args, _rparen] => Call {
          f: f.lexeme,
          args: Self::parse_args(ast, args),
        },
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable: {:?}", node),
    }
  }
}

// TODO: write a better parser-generator. The one we have right now
// loses a lot of semantic information (and just emits the raw tokens
// at each node). This is fine for the e2e api of this file (you give me
// tokens, I give you AST), but it's duplicative.
impl Expression {
  fn from_cst(root: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    if let Node::Parent(t) = node {
      match (t.rule, t.children.as_slice()) {
        (LoxRule::Assign, [lhs, Node::Leaf(eq), rhs]) if eq.token_type == LoxTokenKind::Equal => {
          Expression::Assign(Assign {
            assignee: LValue::from_cst(root, lhs),
            assign: Box::new(Expression::from_cst(root, rhs)),
          })
        }
        (LoxRule::Product | LoxRule::Term | LoxRule::Compare, [lhs, Node::Leaf(op), rhs]) => {
          Expression::Bin(Binary {
            lhs: Box::new(Self::from_cst(root, lhs)),
            op: BinOp::from_token(op.token_type)
              .unwrap_or_else(|| panic!("Unexpected binary op: {}", op.token_type)),
            rhs: Box::new(Self::from_cst(root, rhs)),
          })
        }
        (LoxRule::Paren, [Node::Leaf(lparen), expr, Node::Leaf(rparen)])
          if lparen.token_type == LoxTokenKind::LParen
            && rparen.token_type == LoxTokenKind::RParen =>
        {
          Self::from_cst(root, expr)
        }

        (LoxRule::Unary, [Node::Leaf(op), operand]) => Expression::Unary(Unary {
          operand: Box::new(Self::from_cst(root, operand)),
          op: UnaryOp::from_token(op.token_type)
            .unwrap_or_else(|| panic!("Unexpected unary op: {}", op.token_type)),
        }),

        (
          LoxRule::Unit,
          [
            Node::Parent(Parent {
              rule: LoxRule::Call,
              children: _,
            }),
          ],
        ) => Expression::Call(Call::from_cst(root, &t.children[0])),
        (
          LoxRule::Unit,
          [
            Node::Parent(Parent {
              rule: LoxRule::Literal,
              children,
            }),
          ],
        ) => match children.as_slice() {
          [Node::Leaf(num)] if num.token_type == LoxTokenKind::Number => Expression::Lit(
            Literal::Num(root.lexeme_arena.resolve(&num.lexeme).parse().unwrap()),
          ),
          [Node::Leaf(num)] if num.token_type == LoxTokenKind::True => {
            Expression::Lit(Literal::Bool(true))
          }

          [Node::Leaf(num)] if num.token_type == LoxTokenKind::False => {
            Expression::Lit(Literal::Bool(false))
          }

          [Node::Leaf(ident)] if ident.token_type == LoxTokenKind::Ident => {
            Expression::Lit(Literal::Var(ident.lexeme))
          }
          [Node::Leaf(ident)] if ident.token_type == LoxTokenKind::Nil => Expression::Nil,

          [Node::Leaf(s)] if s.token_type == LoxTokenKind::String => {
            let inner = root
              .lexeme_arena
              .resolve(&s.lexeme)
              .strip_prefix("\"")
              .and_then(|s| s.strip_suffix("\""))
              .unwrap()
              .to_string();

            Expression::Lit(Literal::String(inner))
          }
          _ => panic!("unreachable literal"),
        },

        // passthroughs
        (
          LoxRule::Paren
          | LoxRule::Expr
          | LoxRule::Unary
          | LoxRule::Term
          | LoxRule::Product
          | LoxRule::Compare
          | LoxRule::Assign,
          [x],
        ) => Self::from_cst(root, x),
        _ => panic!("unreachable: {:?}", node),
      }
    } else {
      panic!("unreachable")
    }
  }
}

impl Ast {
  pub fn from_cst(ast: Tree<LoxRule>) -> Self {
    let root = Program::from_cst(&ast, &ast.root);
    Ast {
      root,
      lexeme_arena: ast.lexeme_arena,
    }
  }
}
