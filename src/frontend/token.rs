use crate::core::ordinal::Ordinal;
use crate::lexer::{Result, TokenType};
use crate::{lexer, ordinal_enum};
use lasso::Rodeo;
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::sync::OnceLock;

ordinal_enum!(LoxTokenType {
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
});

impl TokenType for LoxTokenType {
    fn eof() -> Self {
        LoxTokenType::Eof
    }
}

pub type LoxToken = lexer::Token<LoxTokenType>;

static LEXER: OnceLock<lexer::Lexer<LoxTokenType>> = OnceLock::new();

pub fn lex(program: &str, rodeo: &mut Rodeo) -> Result<Vec<LoxToken>> {
    let lexical_spec = vec![
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
    LEXER
        .get_or_init(|| lexer::Lexer::make(lexical_spec).unwrap())
        .lex(program, rodeo)
        .map(|tokens| {
            tokens
                .into_iter()
                .filter(|t| t.token_type != LoxTokenType::WhiteSpace)
                .collect()
        })
}

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

        let tokens: Vec<_> = lex(program, &mut rodeo)
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
