use lasso::Spur;
use lexer::TokenType;
use lox_derive::Ordinal;
use std::collections::VecDeque;
use strum::Display;

pub type Ident = Spur;

pub fn lexer() -> Result<lexer::Lexer<LParseToken>, lexer::Error> {
  lexer::Lexer::<LParseToken>::new(LEX_SPEC)
}

#[derive(Debug)]
pub struct BoundLeaf {
  pub token: Ident,
  pub bind_to: Ident,
}
#[derive(Debug)]
pub struct BoundRule {
  pub rule: Ident,
  pub bind_to: Ident,
}
#[derive(Debug)]
pub enum LNode {
  Leaf(BoundLeaf),
  Rule(BoundRule),
}
#[derive(Debug)]
pub struct ProductionDefinition {
  pub definition: VecDeque<LNode>,
  pub semantic_action: Ident,
}

#[derive(Debug)]
pub struct LRule {
  pub name: Ident,
  pub return_type: Ident,
  pub productions: VecDeque<ProductionDefinition>,
}

#[derive(Debug)]
pub struct LGrammar {
  pub preamble: VecDeque<Spur>,
  pub goal_rule: Ident,
  pub token_type: Ident,
  pub rules: VecDeque<LRule>,
}

#[derive(Ordinal, Eq, PartialEq, Hash, Display, Debug, PartialOrd)]
pub enum LParseToken {
  LCurlyBrace,
  RCurlyBrace,
  LSquareBracket,
  RSquareBracket,
  LAngleBracket,
  RAngleBracket,
  Equals,
  Semicolon,
  Colon,
  Comma,
  RustImport,

  // keywords
  GoalRule,
  TokenType,

  Ident,

  EmbeddedRust,
  Whitespace,
  Eof,
}

impl TokenType for LParseToken {
  fn whitespace() -> Self {
    Self::Whitespace
  }
  fn eof() -> Self {
    Self::Eof
  }
}

const LEX_SPEC: &[(LParseToken, &str)] = &[
  (LParseToken::LCurlyBrace, "{"),
  (LParseToken::RCurlyBrace, "}"),
  (LParseToken::LSquareBracket, "\\["),
  (LParseToken::RSquareBracket, "\\]"),
  (LParseToken::LAngleBracket, "<"),
  (LParseToken::RAngleBracket, ">"),
  (LParseToken::Equals, "="),
  (LParseToken::Semicolon, ";"),
  (LParseToken::Colon, ":"),
  (LParseToken::Comma, ","),
  (LParseToken::GoalRule, "goal_rule"),
  (LParseToken::TokenType, "token_type"),
  (LParseToken::EmbeddedRust, "%[\u{0}-\u{24}\u{26}-\u{7F}]*%"),
  (LParseToken::Whitespace, " "),
  (LParseToken::Whitespace, "\t"),
  (LParseToken::Whitespace, "\n"),
  (LParseToken::RustImport, "use ([a-zA-Z]|_|{|}|:|,| )*;"),
  (LParseToken::Ident, "([a-zA-Z]|_)([a-zA-Z0-9]|_)*"),
];
