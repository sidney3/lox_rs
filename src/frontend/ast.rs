use super::error::{Error, Result};
use super::token::{LoxToken, LoxTokenType};
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
    fn from_token(token: LoxTokenType) -> Option<Self> {
        match token {
            LoxTokenType::Star => Some(Self::Times),
            LoxTokenType::Slash => Some(Self::Divide),
            LoxTokenType::Plus => Some(Self::Plus),
            LoxTokenType::Minus => Some(Self::Minus),
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

impl Expression {
    fn from_cst(node: &Node<LoxRule>) -> Self {
        if let Node::Tree(t) = node {
            match (t.rule, t.children.as_slice()) {
                (LoxRule::Expr | LoxRule::Term, [lhs, Node::Leaf(op), rhs])
                    if matches!(
                        op.token_type,
                        LoxTokenType::Plus
                            | LoxTokenType::Minus
                            | LoxTokenType::Slash
                            | LoxTokenType::Star
                    ) =>
                {
                    Expression::Bin(Binary {
                        lhs: Box::new(Self::from_cst(lhs)),
                        op: BinOp::from_token(op.token_type).unwrap(),
                        rhs: Box::new(Self::from_cst(rhs)),
                    })
                }
                (LoxRule::Paren, [Node::Leaf(lparen), expr, Node::Leaf(rparen)])
                    if lparen.token_type == LoxTokenType::LParen
                        && rparen.token_type == LoxTokenType::RParen =>
                {
                    Self::from_cst(expr)
                }

                (LoxRule::Literal, [Node::Leaf(num)]) if num.token_type == LoxTokenType::Number => {
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
    type TokenType = LoxTokenType;
    fn goal() -> Self {
        LoxRule::Expr
    }
}

type CSTParser = Parser<LoxRule>;
type P = Production<LoxRule>;

fn parser() -> CSTParser {
    let grammar = Grammar::new(vec![
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
                Symbol::Token(LoxTokenType::Plus),
                Symbol::Rule(LoxRule::Term)
            ],
        },
        // Expr := Expr '-' Term
        P {
            rule: LoxRule::Expr,
            definition: nonempty![
                Symbol::Rule(LoxRule::Expr),
                Symbol::Token(LoxTokenType::Minus),
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
                Symbol::Token(LoxTokenType::Star),
                Symbol::Rule(LoxRule::Unary)
            ],
        },
        // Term := Term '/' Unary
        P {
            rule: LoxRule::Term,
            definition: nonempty![
                Symbol::Rule(LoxRule::Term),
                Symbol::Token(LoxTokenType::Slash),
                Symbol::Rule(LoxRule::Unary)
            ],
        },
        // Unary := '-' Paren
        P {
            rule: LoxRule::Unary,
            definition: nonempty![
                Symbol::Token(LoxTokenType::Bang),
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
                Symbol::Token(LoxTokenType::LParen),
                Symbol::Rule(LoxRule::Expr),
                Symbol::Token(LoxTokenType::RParen),
            ],
        },
        // Literal := 'Num'
        P {
            rule: LoxRule::Literal,
            definition: nonempty![Symbol::Token(LoxTokenType::Number)],
        },
    ]);

    CSTParser::new(grammar)
}

pub fn parse(tokens: Vec<LoxToken>) -> Result<Ast> {
    todo!();
}
