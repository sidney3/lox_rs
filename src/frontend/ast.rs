use super::error::Result;
use super::token::LoxTokenKind;
use crate::lexer::Tokens;
use crate::parse::{Grammar, Node, Parent, Parser, Production, Rule, Symbol, Tree};
use lox_derive::Ordinal;
use nonempty::nonempty;
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
    fn from_cst(root: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
        if let Node::Parent(t) = node {
            match (t.rule, t.children.as_slice()) {
                (LoxRule::Product | LoxRule::Term, [lhs, Node::Leaf(op), rhs])
                    if matches!(
                        op.token_type,
                        LoxTokenKind::Plus
                            | LoxTokenKind::Minus
                            | LoxTokenKind::Slash
                            | LoxTokenKind::Star
                    ) =>
                {
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

                // passthroughs
                (
                    LoxRule::Paren
                    | LoxRule::Expr
                    | LoxRule::Unary
                    | LoxRule::Term
                    | LoxRule::Product,
                    [x],
                ) => Self::from_cst(root, x),
                _ => panic!("unreachable"),
            }
        } else {
            panic!("unreachable")
        }
    }
}

pub enum AstNode {
    Expr(Expression),
}

impl AstNode {
    pub fn from_cst(ast: &Tree<LoxRule>, node: &Node<LoxRule>) -> Self {
        match node {
            Node::Parent(Parent { rule, children }) if *rule == LoxRule::Expr => {
                AstNode::Expr(Expression::from_cst(ast, node))
            }
            _ => panic!("unreachable"),
        }
    }
}

pub struct Ast {
    pub lexeme_arena: lasso::Rodeo,
    pub root: AstNode,
}

impl Ast {
    pub fn from_cst(ast: Tree<LoxRule>) -> Self {
        let root = AstNode::from_cst(&ast, &ast.root);
        Ast {
            root,
            lexeme_arena: ast.lexeme_arena,
        }
    }
}

#[derive(Ordinal, Eq, PartialEq, Hash, Display, Debug, PartialOrd)]
pub enum LoxRule {
    Paren,
    Term,
    Product,
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
            definition: nonempty![Symbol::Rule(LoxRule::Term),],
        },
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

    pub fn parse(&self, tokens: Tokens<LoxTokenKind>) -> Result<Ast> {
        let cst = self.0.parse(tokens)?;

        Ok(Ast::from_cst(cst))
    }
}
