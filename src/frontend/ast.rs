use crate::frontend::token::Ident;

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

pub struct Ast {
  pub lexeme_arena: lasso::Rodeo,
  pub root: Program,
}

#[derive(Debug)]
pub enum UnaryOp {
  Minus,
  Not,
}

#[derive(Debug)]
pub struct Binary {
  pub lhs: Box<Expression>,
  pub op: BinOp,
  pub rhs: Box<Expression>,
}

impl Binary {
  pub fn new(lhs: Expression, op: BinOp, rhs: Expression) -> Self {
    Self {
      lhs: Box::new(lhs),
      op,
      rhs: Box::new(rhs),
    }
  }
}

#[derive(Debug)]
pub struct Unary {
  pub operand: Box<Expression>,
  pub op: UnaryOp,
}

impl Unary {
  pub fn new(op: UnaryOp, operand: Expression) -> Self {
    Unary {
      op,
      operand: Box::new(operand),
    }
  }
}

#[derive(Debug)]
pub enum Literal {
  Num(Ident),
  String(Ident),
  Bool(bool),
  Var(Ident),
}

#[derive(Debug)]
pub struct Call {
  pub callee: Box<Expression>,
  pub args: Vec<Expression>,
}

#[derive(Debug)]
pub struct Member {
  pub accessee: Box<Expression>,
  pub property: Ident,
}

// NOTE: these don't all take the same precedence. Once I
// compile in the semantic actions and add codegen for
// nodes should be much better.
#[derive(Debug)]
pub enum Expression {
  Bin(Binary),
  Unary(Unary),
  Lit(Literal),
  Assign(Assign),
  Call(Call),
  SuperMember(Ident),
  Member(Member),
  Nil,
}

#[derive(Debug)]
pub struct ExprStatement {
  pub expr: Expression,
}

#[derive(Debug)]
pub struct WhileStatement {
  pub cond: Expression,
  pub body: Box<Block>,
}

#[derive(Debug)]
pub struct ForLoop {
  pub init: Box<VarDeclaration>,
  pub condition: Expression,
  pub increment: Expression,
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
  Block(Block),
  If(IfStatement),
  While(WhileStatement),
  For(ForLoop),
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
  pub ident: Ident,
  pub assign: Expression,
}

#[derive(Debug)]
pub struct ClassDeclaration {
  pub ident: Ident,
  pub methods: Vec<FuncDecl>,
  pub inherits: Option<Ident>,
}

#[derive(Debug)]
pub enum Declaration {
  Statement(Statement),
  Var(VarDeclaration),
  Fun(FuncDecl),
  Class(ClassDeclaration),
}

#[derive(Debug)]
pub enum LValue {
  Var(lasso::Spur),
  Member(Member),
}

#[derive(Debug)]
pub struct Assign {
  pub assignee: LValue,
  pub assign: Box<Expression>,
}

#[derive(Debug)]
pub struct Program {
  pub declarations: Vec<Declaration>,
}
