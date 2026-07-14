use std::collections::{VecDeque, vec_deque};

use lox_derive::Ordinal;
use nonempty::nonempty;
use strum::Display;

use super::error::Result;
use super::token::LoxTokenKind;
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
  Statement,
  ExprStatement,
}

impl Rule for LoxRule {
  type TokenType = LoxTokenKind;
  fn goal() -> Self {
    LoxRule::Program
  }
}

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
      _ => None,
    }
  }
}

pub struct Ast {
  pub lexeme_arena: lasso::Rodeo,
  pub root: Program,
}

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

pub struct Binary {
  pub lhs: Box<Expression>,
  pub op: BinOp,
  pub rhs: Box<Expression>,
}

pub struct Unary {
  pub operand: Box<Expression>,
  pub op: UnaryOp,
}

pub enum Literal {
  Num(f64),
  String(String),
}

pub enum Expression {
  Bin(Binary),
  Unary(Unary),
  Lit(Literal),
}

pub struct ExprStatement {
  pub expr: Expression,
}

pub struct PrintStatement {
  pub operand: Expression,
}

pub struct AssertStatement {
  pub operand: Expression,
}

pub enum Statement {
  Expr(ExprStatement),
  Print(PrintStatement),
  Assert(AssertStatement),
}

pub enum Declaration {
  Statement(Statement),
}

pub struct Program {
  pub declarations: VecDeque<Declaration>,
}

type P = Production<LoxRule>;

fn lox_grammar() -> Grammar<LoxRule> {
  Grammar::new(vec![
    // Program := Declaration | Program Declaration;
    P {
      rule: LoxRule::Program,
      definition: nonempty![Symbol::Rule(LoxRule::Declaration)],
    },
    P {
      rule: LoxRule::Program,
      definition: nonempty![
        Symbol::Rule(LoxRule::Program),
        Symbol::Rule(LoxRule::Declaration),
      ],
    },
    // Declaration := Statement;
    P {
      rule: LoxRule::Declaration,
      definition: nonempty![Symbol::Rule(LoxRule::Statement)],
    },
    // Statement := AssertStatement | PrintStatement | ExpressionStatemt;
    P {
      rule: LoxRule::Statement,
      definition: nonempty![Symbol::Rule(LoxRule::Print)],
    },
    P {
      rule: LoxRule::Statement,
      definition: nonempty![Symbol::Rule(LoxRule::Assert)],
    },
    P {
      rule: LoxRule::Statement,
      definition: nonempty![Symbol::Rule(LoxRule::ExprStatement)],
    },
    // PrintStatement := 'print' Expr ';'
    P {
      rule: LoxRule::Print,
      definition: nonempty![
        Symbol::Token(LoxTokenKind::Print),
        Symbol::Rule(LoxRule::Expr),
        Symbol::Token(LoxTokenKind::Semicolon)
      ],
    },
    // AssertStatement := 'assert' Expr ';'
    P {
      rule: LoxRule::Assert,
      definition: nonempty![
        Symbol::Token(LoxTokenKind::Assert),
        Symbol::Rule(LoxRule::Expr),
        Symbol::Token(LoxTokenKind::Semicolon)
      ],
    },
    // ExprStatement := Expr ';'
    P {
      rule: LoxRule::ExprStatement,
      definition: nonempty![
        Symbol::Rule(LoxRule::Expr),
        Symbol::Token(LoxTokenKind::Semicolon)
      ],
    },
    // ------------------
    // EXPRESSION GRAMMAR
    // ------------------
    // Expr := Compare
    P {
      rule: LoxRule::Expr,
      definition: nonempty![Symbol::Rule(LoxRule::Compare),],
    },
    // Compare :=
    //       | Term '==' Compare
    //       | Term '>=' Compare
    //       | Term '>' Compare
    //       | Term '<' Compare
    //       | Term '<=' Compare
    //       | Term
    //
    P {
      rule: LoxRule::Compare,
      definition: nonempty![
        Symbol::Rule(LoxRule::Term),
        Symbol::Token(LoxTokenKind::EqualEqual),
        Symbol::Rule(LoxRule::Compare)
      ],
    },
    P {
      rule: LoxRule::Compare,
      definition: nonempty![
        Symbol::Rule(LoxRule::Term),
        Symbol::Token(LoxTokenKind::LessEqual),
        Symbol::Rule(LoxRule::Compare)
      ],
    },
    P {
      rule: LoxRule::Compare,
      definition: nonempty![
        Symbol::Rule(LoxRule::Term),
        Symbol::Token(LoxTokenKind::Less),
        Symbol::Rule(LoxRule::Compare)
      ],
    },
    P {
      rule: LoxRule::Compare,
      definition: nonempty![
        Symbol::Rule(LoxRule::Term),
        Symbol::Token(LoxTokenKind::GreaterEqual),
        Symbol::Rule(LoxRule::Compare)
      ],
    },
    P {
      rule: LoxRule::Compare,
      definition: nonempty![
        Symbol::Rule(LoxRule::Term),
        Symbol::Token(LoxTokenKind::Greater),
        Symbol::Rule(LoxRule::Compare)
      ],
    },
    P {
      rule: LoxRule::Compare,
      definition: nonempty![Symbol::Rule(LoxRule::Term),],
    },
    //
    // Term := Product '+' Term
    //       | Product '-' Term
    //       | Product
    P {
      rule: LoxRule::Term,
      definition: nonempty![
        Symbol::Rule(LoxRule::Term),
        Symbol::Token(LoxTokenKind::Plus),
        Symbol::Rule(LoxRule::Product)
      ],
    },
    P {
      rule: LoxRule::Term,
      definition: nonempty![
        Symbol::Rule(LoxRule::Term),
        Symbol::Token(LoxTokenKind::Minus),
        Symbol::Rule(LoxRule::Product)
      ],
    },
    P {
      rule: LoxRule::Term,
      definition: nonempty![Symbol::Rule(LoxRule::Product)],
    },
    // Product := Unary | Product '*' Unary | Product '/' Unary
    P {
      rule: LoxRule::Product,
      definition: nonempty![Symbol::Rule(LoxRule::Unary)],
    },
    P {
      rule: LoxRule::Product,
      definition: nonempty![
        Symbol::Rule(LoxRule::Product),
        Symbol::Token(LoxTokenKind::Star),
        Symbol::Rule(LoxRule::Unary)
      ],
    },
    P {
      rule: LoxRule::Product,
      definition: nonempty![
        Symbol::Rule(LoxRule::Product),
        Symbol::Token(LoxTokenKind::Slash),
        Symbol::Rule(LoxRule::Unary)
      ],
    },
    // Unary := '-' Unary | Paren
    P {
      rule: LoxRule::Unary,
      definition: nonempty![
        Symbol::Token(LoxTokenKind::Bang),
        Symbol::Rule(LoxRule::Unary),
      ],
    },
    P {
      rule: LoxRule::Unary,
      definition: nonempty![Symbol::Rule(LoxRule::Paren),],
    },
    // Paren := Literal | '(' Expr ')'
    P {
      rule: LoxRule::Paren,
      definition: nonempty![Symbol::Rule(LoxRule::Literal),],
    },
    P {
      rule: LoxRule::Paren,
      definition: nonempty![
        Symbol::Token(LoxTokenKind::LParen),
        Symbol::Rule(LoxRule::Expr),
        Symbol::Token(LoxTokenKind::RParen),
      ],
    },
    // Literal := 'Num' | 'Str'
    P {
      rule: LoxRule::Literal,
      definition: nonempty![Symbol::Token(LoxTokenKind::Number)],
    },
    P {
      rule: LoxRule::Literal,
      definition: nonempty![Symbol::Token(LoxTokenKind::String)],
    },
  ])
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
        rule: LoxRule::Print,
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
            rule: LoxRule::ExprStatement,
            children: _,
          }),
        ] => Statement::Expr(ExprStatement::from_cst(ast, &children[0])),
        _ => panic!(),
      },
      _ => panic!("unreachable: {:?}", node),
    }
  }
}

impl Declaration {
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::Declaration,
        children: children,
      }) => match children.as_slice() {
        [statement] => Declaration::Statement(Statement::from_cst(ast, statement)),
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable: {:?}", node),
    }
  }
}

impl Program {
  pub fn new() -> Self {
    Self {
      declarations: VecDeque::new(),
    }
  }
  pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
    match node {
      Node::Parent(Parent {
        rule: LoxRule::Program,
        children,
      }) => match children.as_slice() {
        [program_node, decl_node] => {
          let decl = Declaration::from_cst(ast, decl_node);
          let mut program = Program::from_cst(ast, program_node);

          program.declarations.push_front(decl);
          program
        }
        [decl_node] => {
          let decl = Declaration::from_cst(ast, decl_node);
          let mut decls = VecDeque::new();
          decls.push_back(decl);
          Program {
            declarations: decls,
          }
        }
        _ => panic!("unreachable"),
      },
      _ => panic!("unreachable"),
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
        (LoxRule::Product | LoxRule::Term | LoxRule::Compare, [lhs, Node::Leaf(op), rhs]) => {
          Expression::Bin(Binary {
            lhs: Box::new(Self::from_cst(root, lhs)),
            op: BinOp::from_token(op.token_type).unwrap(),
            rhs: Box::new(Self::from_cst(root, rhs)),
          })
        }
        (LoxRule::Paren, [Node::Leaf(lparen), expr, Node::Leaf(rparen)])
          if lparen.token_type == LoxTokenKind::LParen
            && rparen.token_type == LoxTokenKind::RParen =>
        {
          Self::from_cst(root, expr)
        }

        (LoxRule::Unary, [Node::Leaf(op), operand])
          if matches!(op.token_type, LoxTokenKind::Minus) =>
        {
          Expression::Unary(Unary {
            operand: Box::new(Self::from_cst(root, operand)),
            op: UnaryOp::from_token(op.token_type).unwrap(),
          })
        }

        (LoxRule::Literal, [Node::Leaf(num)]) if num.token_type == LoxTokenKind::Number => {
          Expression::Lit(Literal::Num(
            root.lexeme_arena.resolve(&num.lexeme).parse().unwrap(),
          ))
        }

        (LoxRule::Literal, [Node::Leaf(s)]) if s.token_type == LoxTokenKind::String => {
          let inner = root
            .lexeme_arena
            .resolve(&s.lexeme)
            .strip_prefix("\"")
            .and_then(|s| s.strip_suffix("\""))
            .unwrap()
            .to_string();

          Expression::Lit(Literal::String(inner))
        }

        // passthroughs
        (
          LoxRule::Paren
          | LoxRule::Expr
          | LoxRule::Unary
          | LoxRule::Term
          | LoxRule::Product
          | LoxRule::Compare,
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
