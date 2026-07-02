use super::error::{Error, Result};
use crate::core::Ordinal;
use crate::lexer;
use crate::lexer::TokenType;
use lasso::Rodeo;
use lox_derive::Ordinal;
use std::hash::Hash;
use std::sync::OnceLock;
use strum::Display;

#[derive(Ordinal, Debug, Display, Hash)]
pub enum LoxTokenType {
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

impl TokenType for LoxTokenType {
    fn eof() -> Self {
        LoxTokenType::Eof
    }
}

pub type LoxToken = lexer::Token<LoxTokenType>;

pub struct LoxLexer(lexer::Lexer<LoxTokenType>);

impl LoxLexer {
    pub fn new() -> Result<Self> {
        Ok(Self(lexer::Lexer::<LoxTokenType>::new(LEXICAL_SPEC)?))
    }

    pub fn lex(&self, program: &str, arena: &mut Rodeo) -> Result<Vec<LoxToken>> {
        let tokens = self.0.lex(program, arena)?;

        let filtered = tokens
            .into_iter()
            .filter(|t| t.token_type != LoxTokenType::WhiteSpace)
            .collect();

        Ok(filtered)
    }
}

const LEXICAL_SPEC: &[(LoxTokenType, &str)] = &[
    (LoxTokenType::WhiteSpace, " "),
    (LoxTokenType::WhiteSpace, "\t"),
    (LoxTokenType::WhiteSpace, "\n"),
    (LoxTokenType::LParen, "\\("),
    (LoxTokenType::RParen, "\\)"),
    (LoxTokenType::LBrace, "\\["),
    (LoxTokenType::RBrace, "\\]"),
    (LoxTokenType::Comma, ","),
    (LoxTokenType::Dot, "\\."),
    (LoxTokenType::Minus, "\\-"),
    (LoxTokenType::Plus, "\\+"),
    (LoxTokenType::Semicolon, ";"),
    (LoxTokenType::Slash, "/"),
    (LoxTokenType::Star, "\\*"),
    (LoxTokenType::Bang, "!"),
    (LoxTokenType::BangEqual, "!="),
    (LoxTokenType::Equal, "="),
    (LoxTokenType::EqualEqual, "=="),
    (LoxTokenType::Greater, ">"),
    (LoxTokenType::GreaterEqual, ">="),
    (LoxTokenType::Less, "<"),
    (LoxTokenType::LessEqual, "<="),
    (LoxTokenType::For, "for"),
    (LoxTokenType::False, "false"),
    (LoxTokenType::And, "and"),
    (LoxTokenType::Class, "class"),
    (LoxTokenType::Else, "else"),
    (LoxTokenType::Fun, "fun"),
    (LoxTokenType::If, "if"),
    (LoxTokenType::Nil, "nil"),
    (LoxTokenType::Or, "or"),
    (LoxTokenType::Return, "return"),
    (LoxTokenType::Super, "super"),
    (LoxTokenType::This, "this"),
    (LoxTokenType::True, "true"),
    (LoxTokenType::Var, "var"),
    (LoxTokenType::While, "while"),
    (LoxTokenType::Print, "print"), // TODO: remove this token
    (LoxTokenType::Number, "[1-9][0-9]*"),
    (LoxTokenType::String, "\"[\u{20}-\u{7E}]*\""),
    (LoxTokenType::Ident, "[a-zA-Z]([a-zA-Z0-9]|_)*"),
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

        let mut rodeo = Rodeo::default();
        let lexer = LoxLexer::new().unwrap();

        let tokens: Vec<_> = lexer
            .lex(program, &mut rodeo)
            .unwrap()
            .into_iter()
            .filter(|token| token.token_type != LoxTokenType::WhiteSpace)
            .collect();

        let mut spur = |s| rodeo.get_or_intern(s);

        assert_eq!(
            tokens,
            vec![
                LoxToken {
                    lexeme: spur("print"),
                    token_type: LoxTokenType::Print,
                    line: 1
                },
                LoxToken {
                    lexeme: spur("("),
                    token_type: LoxTokenType::LParen,
                    line: 1,
                },
                LoxToken {
                    lexeme: spur("\"Hello world!\""),
                    token_type: LoxTokenType::String,
                    line: 1
                },
                LoxToken {
                    lexeme: spur(")"),
                    token_type: LoxTokenType::RParen,
                    line: 1,
                },
                LoxToken {
                    lexeme: spur(";"),
                    token_type: LoxTokenType::Semicolon,
                    line: 1
                },
                LoxToken {
                    lexeme: spur("EOF"),
                    token_type: LoxTokenType::Eof,
                    line: 2,
                }
            ]
        )
    }
}
