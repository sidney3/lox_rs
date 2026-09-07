use super::Production;

use super::Error;
use super::Grammar;
use super::Parser;
use super::Rule;
use super::Symbol;
use lasso::{Rodeo, Spur};
use lexer::TokenType;
use lexer::Tokens;
use lox_derive::Ordinal;
use strum::Display;

pub type Ident = Spur;

pub fn lexer() -> Result<lexer::Lexer<LParseToken>, lexer::Error> {
  lexer::Lexer::<LParseToken>::new(LEX_SPEC)
}

pub struct LParseParser {
  parser: Parser<LParseRule>,
}

pub trait ParseLParseExt {
  fn parse_lparse(&self, tokens: Tokens<LParseToken>) -> Result<(Rodeo, LGrammar), Error>;
}

pub fn parser() -> Parser<LParseRule> {
  Parser::new(lparse_grammar())
}
impl ParseLParseExt for Parser<LParseRule> {
  fn parse_lparse(&self, tokens: Tokens<LParseToken>) -> Result<(Rodeo, LGrammar), Error> {
    let cst = self.parse(tokens)?;

    Ok((cst.lexeme_arena, parse_lparse_grammar(&cst.root)))
  }
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
  pub definition: Vec<LNode>,
  pub semantic_action: Ident,
}

#[derive(Debug)]
pub struct LRule {
  pub name: Ident,
  pub return_type: Ident,
  pub productions: Vec<ProductionDefinition>,
}

#[derive(Debug)]
pub struct LGrammar {
  pub preamble: Vec<Spur>,
  pub goal_rule: Ident,
  pub token_type: Ident,
  pub rules: Vec<LRule>,
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

#[derive(Ordinal, Eq, PartialEq, Hash, Display, Debug, PartialOrd)]
pub enum LParseRule {
  BoundLeaf,
  BoundRule,
  Node,
  Nodes,
  ProductionDefinition,
  ProductionDefinitions,
  Rule,
  Rules,
  SetGoalRule,
  SetTokenType,
  Preamble,
  Grammar,
}

impl Rule for LParseRule {
  type TokenType = LParseToken;
}

// See docs/parse_grammar.md
fn lparse_grammar() -> Grammar<LParseRule> {
  type P = Production<LParseRule>;
  Grammar::new(
    LParseRule::Grammar, //<goal rule
    vec![
      P {
        rule: LParseRule::BoundLeaf,
        definition: vec![
          Symbol::Token(LParseToken::LSquareBracket),
          Symbol::Token(LParseToken::Ident),
          Symbol::Token(LParseToken::Colon),
          Symbol::Token(LParseToken::Ident),
          Symbol::Token(LParseToken::RSquareBracket),
        ],
      },
      P {
        rule: LParseRule::BoundRule,
        definition: vec![
          Symbol::Token(LParseToken::LAngleBracket),
          Symbol::Token(LParseToken::Ident),
          Symbol::Token(LParseToken::Colon),
          Symbol::Token(LParseToken::Ident),
          Symbol::Token(LParseToken::RAngleBracket),
        ],
      },
      P {
        rule: LParseRule::Node,
        definition: vec![Symbol::Rule(LParseRule::BoundLeaf)],
      },
      P {
        rule: LParseRule::Node,
        definition: vec![Symbol::Rule(LParseRule::BoundRule)],
      },
      P {
        rule: LParseRule::Nodes,
        definition: vec![],
      },
      P {
        rule: LParseRule::Nodes,
        definition: vec![
          Symbol::Rule(LParseRule::Node),
          Symbol::Rule(LParseRule::Nodes),
        ],
      },
      P {
        rule: LParseRule::ProductionDefinition,
        definition: vec![
          Symbol::Rule(LParseRule::Nodes),
          Symbol::Token(LParseToken::Equals),
          Symbol::Token(LParseToken::RAngleBracket),
          Symbol::Token(LParseToken::EmbeddedRust),
          Symbol::Token(LParseToken::Comma),
        ],
      },
      P {
        rule: LParseRule::ProductionDefinitions,
        definition: vec![],
      },
      P {
        rule: LParseRule::ProductionDefinitions,
        definition: vec![
          Symbol::Rule(LParseRule::ProductionDefinition),
          Symbol::Rule(LParseRule::ProductionDefinitions),
        ],
      },
      P {
        rule: LParseRule::Rule,
        definition: vec![
          Symbol::Token(LParseToken::Ident),
          Symbol::Token(LParseToken::Colon),
          Symbol::Token(LParseToken::EmbeddedRust),
          Symbol::Token(LParseToken::LCurlyBrace),
          Symbol::Rule(LParseRule::ProductionDefinitions),
          Symbol::Token(LParseToken::RCurlyBrace),
          Symbol::Token(LParseToken::Semicolon),
        ],
      },
      P {
        rule: LParseRule::Rules,
        definition: vec![],
      },
      P {
        rule: LParseRule::Rules,
        definition: vec![
          Symbol::Rule(LParseRule::Rule),
          Symbol::Rule(LParseRule::Rules),
        ],
      },
      P {
        rule: LParseRule::SetGoalRule,
        definition: vec![
          Symbol::Token(LParseToken::GoalRule),
          Symbol::Token(LParseToken::Equals),
          Symbol::Token(LParseToken::Ident),
          Symbol::Token(LParseToken::Semicolon),
        ],
      },
      P {
        rule: LParseRule::SetTokenType,
        definition: vec![
          Symbol::Token(LParseToken::TokenType),
          Symbol::Token(LParseToken::Equals),
          Symbol::Token(LParseToken::Ident),
          Symbol::Token(LParseToken::Semicolon),
        ],
      },
      P {
        rule: LParseRule::Preamble,
        definition: vec![],
      },
      P {
        rule: LParseRule::Preamble,
        definition: vec![
          Symbol::Token(LParseToken::RustImport),
          Symbol::Rule(LParseRule::Preamble),
        ],
      },
      P {
        rule: LParseRule::Grammar,
        definition: vec![
          Symbol::Rule(LParseRule::Preamble),
          Symbol::Rule(LParseRule::SetGoalRule),
          Symbol::Rule(LParseRule::SetTokenType),
          Symbol::Rule(LParseRule::Rules),
        ],
      },
    ],
  )
}

type RawNode = super::Node<LParseRule>;
type ParentNode = super::Parent<LParseRule>;

fn parse_bound_leaf(node: &ParentNode) -> BoundLeaf {
  match (&node.rule, node.children.as_slice()) {
    (
      LParseRule::BoundLeaf,
      [
        RawNode::Leaf(l_square_bracket),
        RawNode::Leaf(bound_ident),
        RawNode::Leaf(colon),
        RawNode::Leaf(token_type_ident),
        RawNode::Leaf(r_square_bracket),
      ],
    ) if (l_square_bracket.token_type == LParseToken::LSquareBracket
      && bound_ident.token_type == LParseToken::Ident
      && colon.token_type == LParseToken::Colon
      && token_type_ident.token_type == LParseToken::Ident
      && r_square_bracket.token_type == LParseToken::RSquareBracket) =>
    {
      BoundLeaf {
        token: token_type_ident.lexeme,
        bind_to: bound_ident.lexeme,
      }
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_bound_rule(node: &ParentNode) -> BoundRule {
  match (&node.rule, node.children.as_slice()) {
    (
      LParseRule::BoundRule,
      [
        RawNode::Leaf(l_angle_bracket),
        RawNode::Leaf(bound_ident),
        RawNode::Leaf(colon),
        RawNode::Leaf(rule_ident),
        RawNode::Leaf(r_angle_bracket),
      ],
    ) if (l_angle_bracket.token_type == LParseToken::LAngleBracket
      && bound_ident.token_type == LParseToken::Ident
      && colon.token_type == LParseToken::Colon
      && rule_ident.token_type == LParseToken::Ident
      && r_angle_bracket.token_type == LParseToken::RAngleBracket) =>
    {
      BoundRule {
        rule: rule_ident.lexeme,
        bind_to: bound_ident.lexeme,
      }
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_node(node: &ParentNode) -> LNode {
  match (&node.rule, node.children.as_slice()) {
    (LParseRule::Node, [RawNode::Parent(inner)]) if inner.rule == LParseRule::BoundLeaf => {
      LNode::Leaf(parse_bound_leaf(inner))
    }
    (LParseRule::Node, [RawNode::Parent(inner)]) if inner.rule == LParseRule::BoundRule => {
      LNode::Rule(parse_bound_rule(inner))
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_nodes(node: &ParentNode) -> Vec<LNode> {
  match (&node.rule, node.children.as_slice()) {
    (LParseRule::Nodes, []) => vec![],
    (LParseRule::Nodes, [RawNode::Parent(node_inner), RawNode::Parent(nodes_inner)])
      if (node_inner.rule == LParseRule::Node && nodes_inner.rule == LParseRule::Nodes) =>
    {
      let mut result = vec![parse_node(node_inner)];
      result.extend(parse_nodes(nodes_inner));
      result
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_production_definition(node: &ParentNode) -> ProductionDefinition {
  match (&node.rule, node.children.as_slice()) {
    (
      LParseRule::ProductionDefinition,
      [
        RawNode::Parent(nodes_inner),
        RawNode::Leaf(equals),
        RawNode::Leaf(r_angle_bracket),
        RawNode::Leaf(embedded_rust),
        RawNode::Leaf(comma),
      ],
    ) if (nodes_inner.rule == LParseRule::Nodes
      && equals.token_type == LParseToken::Equals
      && r_angle_bracket.token_type == LParseToken::RAngleBracket
      && embedded_rust.token_type == LParseToken::EmbeddedRust
      && comma.token_type == LParseToken::Comma) =>
    {
      ProductionDefinition {
        definition: parse_nodes(nodes_inner),
        semantic_action: embedded_rust.lexeme,
      }
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_production_definitions(node: &ParentNode) -> Vec<ProductionDefinition> {
  match (&node.rule, node.children.as_slice()) {
    (LParseRule::ProductionDefinitions, []) => vec![],
    (
      LParseRule::ProductionDefinitions,
      [
        RawNode::Parent(definition_inner),
        RawNode::Parent(definitions_inner),
      ],
    ) if (definition_inner.rule == LParseRule::ProductionDefinition
      && definitions_inner.rule == LParseRule::ProductionDefinitions) =>
    {
      let mut result = vec![parse_production_definition(definition_inner)];
      result.extend(parse_production_definitions(definitions_inner));
      result
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_rule(node: &ParentNode) -> LRule {
  match (&node.rule, node.children.as_slice()) {
    (
      LParseRule::Rule,
      [
        RawNode::Leaf(name_ident),
        RawNode::Leaf(colon),
        RawNode::Leaf(return_type_rust_type),
        RawNode::Leaf(l_curly_brace),
        RawNode::Parent(production_definitions_inner),
        RawNode::Leaf(r_curly_brace),
        RawNode::Leaf(semicolon),
      ],
    ) if (name_ident.token_type == LParseToken::Ident
      && colon.token_type == LParseToken::Colon
      && return_type_rust_type.token_type == LParseToken::EmbeddedRust
      && l_curly_brace.token_type == LParseToken::LCurlyBrace
      && production_definitions_inner.rule == LParseRule::ProductionDefinitions
      && r_curly_brace.token_type == LParseToken::RCurlyBrace
      && semicolon.token_type == LParseToken::Semicolon) =>
    {
      LRule {
        name: name_ident.lexeme,
        return_type: return_type_rust_type.lexeme,
        productions: parse_production_definitions(production_definitions_inner),
      }
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_rules(node: &ParentNode) -> Vec<LRule> {
  match (&node.rule, node.children.as_slice()) {
    (LParseRule::Rules, []) => vec![],
    (LParseRule::Rules, [RawNode::Parent(first), RawNode::Parent(rules_inner)])
      if (first.rule == LParseRule::Rule && rules_inner.rule == LParseRule::Rules) =>
    {
      let mut result = parse_rules(rules_inner);
      result.push(parse_rule(first));
      result
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_set_goal_rule(node: &ParentNode) -> Ident {
  match (&node.rule, node.children.as_slice()) {
    (
      LParseRule::SetGoalRule,
      [
        RawNode::Leaf(goal_rule_kw),
        RawNode::Leaf(equals),
        RawNode::Leaf(ident),
        RawNode::Leaf(semicolon),
      ],
    ) if (goal_rule_kw.token_type == LParseToken::GoalRule
      && equals.token_type == LParseToken::Equals
      && ident.token_type == LParseToken::Ident
      && semicolon.token_type == LParseToken::Semicolon) =>
    {
      ident.lexeme
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_set_token_type(node: &ParentNode) -> Ident {
  match (&node.rule, node.children.as_slice()) {
    (
      LParseRule::SetTokenType,
      [
        RawNode::Leaf(token_type_kw),
        RawNode::Leaf(equals),
        RawNode::Leaf(ident),
        RawNode::Leaf(semicolon),
      ],
    ) if (token_type_kw.token_type == LParseToken::TokenType
      && equals.token_type == LParseToken::Equals
      && ident.token_type == LParseToken::Ident
      && semicolon.token_type == LParseToken::Semicolon) =>
    {
      ident.lexeme
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_preamble(node: &ParentNode) -> Vec<Spur> {
  match (&node.rule, node.children.as_slice()) {
    (LParseRule::Preamble, []) => Vec::new(),
    (LParseRule::Preamble, [RawNode::Leaf(first_import), RawNode::Parent(rest)]) => {
      let mut all = parse_preamble(rest);
      all.push(first_import.lexeme);
      all
    }
    _ => panic!("unreachable"),
  }
}

fn parse_grammar(node: &ParentNode) -> LGrammar {
  match (&node.rule, node.children.as_slice()) {
    (
      LParseRule::Grammar,
      [
        RawNode::Parent(preamble),
        RawNode::Parent(set_goal_rule_inner),
        RawNode::Parent(set_token_type_inner),
        RawNode::Parent(rules_inner),
      ],
    ) if (set_goal_rule_inner.rule == LParseRule::SetGoalRule
      && set_token_type_inner.rule == LParseRule::SetTokenType
      && rules_inner.rule == LParseRule::Rules) =>
    {
      LGrammar {
        preamble: parse_preamble(preamble),
        goal_rule: parse_set_goal_rule(set_goal_rule_inner),
        token_type: parse_set_token_type(set_token_type_inner),
        rules: parse_rules(rules_inner),
      }
    }
    _ => panic!("Unreachable"),
  }
}

fn parse_lparse_grammar(node: &RawNode) -> LGrammar {
  if let RawNode::Parent(grammar) = node {
    parse_grammar(grammar)
  } else {
    panic!("Unreachable")
  }
}
