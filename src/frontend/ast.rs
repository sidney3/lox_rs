use super::error::{Error, Result};
use super::token::{LoxToken, LoxTokenKind};
use crate::core::Ordinal;
use crate::parse::{Grammar, Node, Parser, Production, Rule, Symbol, Tree};
use lasso::Spur;
use lox_derive::Ordinal;
use nonempty::nonempty;
use std::sync::OnceLock;
use strum::Display;

pub enum BinOp {
    Times,
    Divide,
    Plus,
    Minus,
}

impl BinOp {
    fn from_token(token: LoxTokenKind) -> Option<Self> {
        match token {
            LoxTokenKind::Star => Some(Self::Times),
            LoxTokenKind::Slash => Some(Self::Divide),
            LoxTokenKind::Plus => Some(Self::Plus),
            LoxTokenKind::Minus => Some(Self::Minus),
            _ => None,
        }
    }
}

pub struct Binary {
    pub lhs: Box<Expression>,
    pub op: BinOp,
    pub rhs: Box<Expression>,
}

pub enum UnaryOp {
    Minus,
}

pub struct Unary {
    pub operand: Box<Expression>,
    pub op: UnaryOp,
}

pub enum Literal {
    Num(Spur),
}

pub enum Expression {
    Bin(Binary),
    Unary(Unary),
    Lit(Literal),
}

// TODO: write a better parser-generator. The one we have right now
// loses a lot of semantic information (and just emits the raw tokens
// at each node). This is fine for the e2e api of this file (you give me
// tokens, I give you AST), but it's duplicative.
impl Expression {
    fn from_cst(node: &Node<LoxRule>) -> Self {
        if let Node::Tree(t) = node {
            match (t.rule, t.children.as_slice()) {
                (LoxRule::Expr | LoxRule::Term, [lhs, Node::Leaf(op), rhs])
                    if matches!(
                        op.token_type,
                        LoxTokenKind::Plus
                            | LoxTokenKind::Minus
                            | LoxTokenKind::Slash
                            | LoxTokenKind::Star
                    ) =>
                {
                    Expression::Bin(Binary {
                        lhs: Box::new(Self::from_cst(lhs)),
                        op: BinOp::from_token(op.token_type).unwrap(),
                        rhs: Box::new(Self::from_cst(rhs)),
                    })
                }
                (LoxRule::Paren, [Node::Leaf(lparen), expr, Node::Leaf(rparen)])
                    if lparen.token_type == LoxTokenKind::LParen
                        && rparen.token_type == LoxTokenKind::RParen =>
                {
                    Self::from_cst(expr)
                }

                (LoxRule::Literal, [Node::Leaf(num)]) if num.token_type == LoxTokenKind::Number => {
                    Expression::Lit(Literal::Num(num.lexeme))
                }

                // passthroughs
                (LoxRule::Paren | LoxRule::Expr | LoxRule::Unary | LoxRule::Term, [x]) => {
                    Self::from_cst(x)
                }
                _ => panic!("unreachable"),
            }
        } else {
            panic!("unreachable")
        }
    }
}

pub enum Ast {
    Expr(Expression),
}

impl Ast {
    pub fn from_cst(node: Tree<LoxRule>) -> Self {
        let rule = node.rule;

        match rule {
            LoxRule::Expr => Ast::Expr(Expression::from_cst(&Node::Tree(node))),
            _ => panic!("unreachable"),
        }
    }
}

#[derive(Ordinal, Hash, Display, Debug, PartialOrd)]
enum LoxRule {
    Paren,
    Term,
    Expr,
    Literal,
    Unary,
}

impl Rule for LoxRule {
    type TokenType = LoxTokenKind;
    fn goal() -> Self {
        LoxRule::Expr
    }
}

type P = Production<LoxRule>;

fn lox_grammar() -> Grammar<LoxRule> {
    Grammar::new(vec![
        // Expr := Term
        P {
            rule: LoxRule::Expr,
            definition: nonempty![Symbol::Rule(LoxRule::Term)],
        },
        // Expr := Expr '+' Term
        P {
            rule: LoxRule::Expr,
            definition: nonempty![
                Symbol::Rule(LoxRule::Expr),
                Symbol::Token(LoxTokenKind::Plus),
                Symbol::Rule(LoxRule::Term)
            ],
        },
        // Expr := Expr '-' Term
        P {
            rule: LoxRule::Expr,
            definition: nonempty![
                Symbol::Rule(LoxRule::Expr),
                Symbol::Token(LoxTokenKind::Minus),
                Symbol::Rule(LoxRule::Term)
            ],
        },
        // Term := Unary
        P {
            rule: LoxRule::Term,
            definition: nonempty![Symbol::Rule(LoxRule::Unary)],
        },
        // Term := Term '*' Unary
        P {
            rule: LoxRule::Term,
            definition: nonempty![
                Symbol::Rule(LoxRule::Term),
                Symbol::Token(LoxTokenKind::Star),
                Symbol::Rule(LoxRule::Unary)
            ],
        },
        // Term := Term '/' Unary
        P {
            rule: LoxRule::Term,
            definition: nonempty![
                Symbol::Rule(LoxRule::Term),
                Symbol::Token(LoxTokenKind::Slash),
                Symbol::Rule(LoxRule::Unary)
            ],
        },
        // Unary := '-' Paren
        P {
            rule: LoxRule::Unary,
            definition: nonempty![
                Symbol::Token(LoxTokenKind::Bang),
                Symbol::Rule(LoxRule::Paren),
            ],
        },
        // Paren := Literal
        P {
            rule: LoxRule::Paren,
            definition: nonempty![Symbol::Rule(LoxRule::Literal),],
        },
        // Paren := '(' Expr ')'
        P {
            rule: LoxRule::Paren,
            definition: nonempty![
                Symbol::Token(LoxTokenKind::LParen),
                Symbol::Rule(LoxRule::Expr),
                Symbol::Token(LoxTokenKind::RParen),
            ],
        },
        // Literal := 'Num'
        P {
            rule: LoxRule::Literal,
            definition: nonempty![Symbol::Token(LoxTokenKind::Number)],
        },
    ])
}

pub struct LoxParser(Parser<LoxRule>);

impl LoxParser {
    pub fn new() -> Self {
        Self(Parser::<LoxRule>::new(lox_grammar()))
    }

    pub fn parse(&self, tokens: Vec<LoxToken>) -> Result<Ast> {
        let cst = self.0.parse(tokens.into_iter())?;

        Ok(Ast::from_cst(cst))
    }
}
