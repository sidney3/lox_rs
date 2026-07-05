use super::error::Result;
use crate::lexer;
use crate::lexer::{TokenType, Tokens};
use lox_derive::Ordinal;
use std::hash::Hash;
use strum::Display;

#[derive(Ordinal, Eq, PartialEq, Debug, Display, Hash)]
pub enum LoxTokenKind {
    WhiteSpace,

    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Keywords
    False,
    And,
    Class,
    Else,
    Fun,
    For,
    If,
    Nil,
    Or,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Print,

    // Literals
    Number,
    String,

    // Other
    Ident,

    Eof,
}

impl TokenType for LoxTokenKind {
    fn eof() -> Self {
        LoxTokenKind::Eof
    }
}

pub type LoxToken = lexer::Token<LoxTokenKind>;

pub struct LoxLexer(lexer::Lexer<LoxTokenKind>);

impl LoxLexer {
    pub fn new() -> Result<Self> {
        Ok(Self(lexer::Lexer::<LoxTokenKind>::new(LEXICAL_SPEC)?))
    }

    pub fn lex(&self, program: &str) -> Result<Tokens<LoxTokenKind>> {
        let result = self.0.lex(program)?;

        let filtered = result
            .tokens
            .into_iter()
            .filter(|t| t.token_type != LoxTokenKind::WhiteSpace)
            .collect();

        Ok(Tokens {
            tokens: filtered,
            lexeme_arena: result.lexeme_arena,
        })
    }
}

const LEXICAL_SPEC: &[(LoxTokenKind, &str)] = &[
    (LoxTokenKind::WhiteSpace, " "),
    (LoxTokenKind::WhiteSpace, "\t"),
    (LoxTokenKind::WhiteSpace, "\n"),
    (LoxTokenKind::LParen, "\\("),
    (LoxTokenKind::RParen, "\\)"),
    (LoxTokenKind::LBrace, "\\["),
    (LoxTokenKind::RBrace, "\\]"),
    (LoxTokenKind::Comma, ","),
    (LoxTokenKind::Dot, "\\."),
    (LoxTokenKind::Minus, "\\-"),
    (LoxTokenKind::Plus, "\\+"),
    (LoxTokenKind::Semicolon, ";"),
    (LoxTokenKind::Slash, "/"),
    (LoxTokenKind::Star, "\\*"),
    (LoxTokenKind::Bang, "!"),
    (LoxTokenKind::BangEqual, "!="),
    (LoxTokenKind::Equal, "="),
    (LoxTokenKind::EqualEqual, "=="),
    (LoxTokenKind::Greater, ">"),
    (LoxTokenKind::GreaterEqual, ">="),
    (LoxTokenKind::Less, "<"),
    (LoxTokenKind::LessEqual, "<="),
    (LoxTokenKind::For, "for"),
    (LoxTokenKind::False, "false"),
    (LoxTokenKind::And, "and"),
    (LoxTokenKind::Class, "class"),
    (LoxTokenKind::Else, "else"),
    (LoxTokenKind::Fun, "fun"),
    (LoxTokenKind::If, "if"),
    (LoxTokenKind::Nil, "nil"),
    (LoxTokenKind::Or, "or"),
    (LoxTokenKind::Return, "return"),
    (LoxTokenKind::Super, "super"),
    (LoxTokenKind::This, "this"),
    (LoxTokenKind::True, "true"),
    (LoxTokenKind::Var, "var"),
    (LoxTokenKind::While, "while"),
    (LoxTokenKind::Print, "print"), // TODO: remove this token
    (LoxTokenKind::Number, "[1-9][0-9]*"),
    (LoxTokenKind::String, "\"[\u{20}-\u{7E}]*\""),
    (LoxTokenKind::Ident, "[a-zA-Z]([a-zA-Z0-9]|_)*"),
];

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_program() {
        let program = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/hello_world.lox"
        ));

        let lexer = LoxLexer::new().unwrap();

        let mut tokens = lexer.lex(program).unwrap();

        let mut spur = |s| tokens.lexeme_arena.get_or_intern(s);

        assert_eq!(
            tokens.tokens,
            vec![
                LoxToken {
                    lexeme: spur("print"),
                    token_type: LoxTokenKind::Print,
                    line: 1
                },
                LoxToken {
                    lexeme: spur("("),
                    token_type: LoxTokenKind::LParen,
                    line: 1,
                },
                LoxToken {
                    lexeme: spur("\"Hello world!\""),
                    token_type: LoxTokenKind::String,
                    line: 1
                },
                LoxToken {
                    lexeme: spur(")"),
                    token_type: LoxTokenKind::RParen,
                    line: 1,
                },
                LoxToken {
                    lexeme: spur(";"),
                    token_type: LoxTokenKind::Semicolon,
                    line: 1
                },
                LoxToken {
                    lexeme: spur("EOF"),
                    token_type: LoxTokenKind::Eof,
                    line: 2,
                }
            ]
        )
    }
}
