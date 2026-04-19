use crate::lexer;
use lasso::Rodeo;
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::sync::OnceLock;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum TokenType {
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
}

type Token<'a> = lexer::Token<TokenType>;

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

static LEXER: OnceLock<lexer::Lexer<TokenType>> = OnceLock::new();

fn lexer() -> &'static lexer::Lexer<TokenType> {
    let lexical_spec = vec![
        (TokenType::WhiteSpace, " "),
        (TokenType::WhiteSpace, "\t"),
        (TokenType::WhiteSpace, "\n"),
        (TokenType::LParen, "\\("),
        (TokenType::RParen, "\\)"),
        (TokenType::LBrace, "\\["),
        (TokenType::RBrace, "\\]"),
        (TokenType::Comma, ","),
        (TokenType::Dot, "\\."),
        (TokenType::Minus, "\\-"),
        (TokenType::Plus, "\\+"),
        (TokenType::Semicolon, ";"),
        (TokenType::Slash, "/"),
        (TokenType::Star, "\\*"),
        (TokenType::Bang, "!"),
        (TokenType::BangEqual, "!="),
        (TokenType::Equal, "="),
        (TokenType::EqualEqual, "=="),
        (TokenType::Greater, ">"),
        (TokenType::GreaterEqual, ">="),
        (TokenType::Less, "<"),
        (TokenType::LessEqual, "<="),
        (TokenType::For, "for"),
        (TokenType::False, "false"),
        (TokenType::And, "and"),
        (TokenType::Class, "class"),
        (TokenType::Else, "else"),
        (TokenType::Fun, "fun"),
        (TokenType::If, "if"),
        (TokenType::Nil, "nil"),
        (TokenType::Or, "or"),
        (TokenType::Return, "return"),
        (TokenType::Super, "super"),
        (TokenType::This, "this"),
        (TokenType::True, "true"),
        (TokenType::Var, "var"),
        (TokenType::While, "while"),
        (TokenType::Print, "print"), // TODO: remove this token
        (TokenType::Number, "[1-9][0-9]*"),
        (TokenType::String, "\"[\u{20}-\u{7E}]*\""),
        (TokenType::Ident, "[a-zA-Z]([a-zA-Z0-9]|_)*"),
    ];
    LEXER.get_or_init(|| lexer::Lexer::make(lexical_spec).unwrap())
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

        let tokens: Vec<_> = lexer()
            .lex(program, &mut rodeo)
            .unwrap()
            .into_iter()
            .filter(|token| token.token_type != TokenType::WhiteSpace)
            .collect();

        let mut spur = |s| rodeo.get_or_intern(s);

        assert_eq!(
            tokens,
            vec![
                Token {
                    lexeme: spur("print"),
                    token_type: TokenType::Print,
                    line: 1
                },
                Token {
                    lexeme: spur("("),
                    token_type: TokenType::LParen,
                    line: 1,
                },
                Token {
                    lexeme: spur("\"Hello world!\""),
                    token_type: TokenType::String,
                    line: 1
                },
                Token {
                    lexeme: spur(")"),
                    token_type: TokenType::RParen,
                    line: 1,
                },
                Token {
                    lexeme: spur(";"),
                    token_type: TokenType::Semicolon,
                    line: 1
                },
            ]
        )
    }
}
