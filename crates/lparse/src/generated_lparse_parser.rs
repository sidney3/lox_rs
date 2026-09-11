#![allow(clippy::all)]
use crate::lparse_frontend::BoundLeaf;
use crate::lparse_frontend::BoundRule;
use crate::lparse_frontend::Ident;
use crate::lparse_frontend::LGrammar;
use crate::lparse_frontend::LNode;
use crate::lparse_frontend::LParseToken;
use crate::lparse_frontend::LRule;
use crate::lparse_frontend::ProductionDefinition;
use lasso::Rodeo;
use lexer::Tokens;
use lox_derive::Ordinal;
use strum::Display;
use lparse;
#[derive(Ordinal, Eq, PartialEq, Hash, Display, Debug, PartialOrd)]
enum ParseRule {
    BoundLeaf,
    BoundRule,
    Node,
    Nodes,
    Production,
    Productions,
    Rule,
    Rules,
    SetGoalRule,
    SetTokenType,
    Preamble,
    Grammar,
}
impl lparse::Rule for ParseRule {
    type TokenType = LParseToken;
}
pub struct LParseParser {
    parser: lparse::Parser<ParseRule>,
}
impl LParseParser {
    pub fn new() -> Self {
        Self {
            parser: lparse::Parser::new(__make_grammar()),
        }
    }
    pub fn parse(
        &self,
        tokens: Tokens<LParseToken>,
    ) -> Result<(Rodeo, LGrammar), lparse::Error> {
        let cst = self.parser.parse(tokens)?;
        if let lparse::Node::Parent(root) = &cst.root {
            Ok((cst.lexeme_arena, __rule_factory_function_grammar(&root)))
        } else {
            panic!("Unreachable, root of CST is a token");
        }
    }
}
fn __make_grammar() -> lparse::Grammar<ParseRule> {
    lparse::Grammar::new(
        ParseRule::Grammar,
        Vec::from([
            lparse::Production::<ParseRule> {
                rule: ParseRule::BoundLeaf,
                definition: Vec::from([
                    lparse::Symbol::Token(LParseToken::LSquareBracket),
                    lparse::Symbol::Token(LParseToken::Ident),
                    lparse::Symbol::Token(LParseToken::Colon),
                    lparse::Symbol::Token(LParseToken::Ident),
                    lparse::Symbol::Token(LParseToken::RSquareBracket),
                ]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::BoundRule,
                definition: Vec::from([
                    lparse::Symbol::Token(LParseToken::LAngleBracket),
                    lparse::Symbol::Token(LParseToken::Ident),
                    lparse::Symbol::Token(LParseToken::Colon),
                    lparse::Symbol::Token(LParseToken::Ident),
                    lparse::Symbol::Token(LParseToken::RAngleBracket),
                ]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Node,
                definition: Vec::from([lparse::Symbol::Rule(ParseRule::BoundLeaf)]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Node,
                definition: Vec::from([lparse::Symbol::Rule(ParseRule::BoundRule)]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Nodes,
                definition: Vec::from([]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Nodes,
                definition: Vec::from([
                    lparse::Symbol::Rule(ParseRule::Nodes),
                    lparse::Symbol::Rule(ParseRule::Node),
                ]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Production,
                definition: Vec::from([
                    lparse::Symbol::Rule(ParseRule::Nodes),
                    lparse::Symbol::Token(LParseToken::Equals),
                    lparse::Symbol::Token(LParseToken::RAngleBracket),
                    lparse::Symbol::Token(LParseToken::EmbeddedRust),
                    lparse::Symbol::Token(LParseToken::Comma),
                ]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Productions,
                definition: Vec::from([]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Productions,
                definition: Vec::from([
                    lparse::Symbol::Rule(ParseRule::Productions),
                    lparse::Symbol::Rule(ParseRule::Production),
                ]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Rule,
                definition: Vec::from([
                    lparse::Symbol::Token(LParseToken::Ident),
                    lparse::Symbol::Token(LParseToken::Colon),
                    lparse::Symbol::Token(LParseToken::EmbeddedRust),
                    lparse::Symbol::Token(LParseToken::LCurlyBrace),
                    lparse::Symbol::Rule(ParseRule::Productions),
                    lparse::Symbol::Token(LParseToken::RCurlyBrace),
                    lparse::Symbol::Token(LParseToken::Semicolon),
                ]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Rules,
                definition: Vec::from([]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Rules,
                definition: Vec::from([
                    lparse::Symbol::Rule(ParseRule::Rules),
                    lparse::Symbol::Rule(ParseRule::Rule),
                ]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::SetGoalRule,
                definition: Vec::from([
                    lparse::Symbol::Token(LParseToken::GoalRule),
                    lparse::Symbol::Token(LParseToken::Equals),
                    lparse::Symbol::Token(LParseToken::Ident),
                    lparse::Symbol::Token(LParseToken::Semicolon),
                ]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::SetTokenType,
                definition: Vec::from([
                    lparse::Symbol::Token(LParseToken::TokenType),
                    lparse::Symbol::Token(LParseToken::Equals),
                    lparse::Symbol::Token(LParseToken::Ident),
                    lparse::Symbol::Token(LParseToken::Semicolon),
                ]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Preamble,
                definition: Vec::from([]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Preamble,
                definition: Vec::from([
                    lparse::Symbol::Rule(ParseRule::Preamble),
                    lparse::Symbol::Token(LParseToken::RustImport),
                ]),
            },
            lparse::Production::<ParseRule> {
                rule: ParseRule::Grammar,
                definition: Vec::from([
                    lparse::Symbol::Rule(ParseRule::Preamble),
                    lparse::Symbol::Rule(ParseRule::SetGoalRule),
                    lparse::Symbol::Rule(ParseRule::SetTokenType),
                    lparse::Symbol::Rule(ParseRule::Rules),
                ]),
            },
        ]),
    )
}
fn __rule_factory_function_bound_leaf(node: &lparse::Parent<ParseRule>) -> BoundLeaf {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::BoundLeaf,
            [lparse::Node::Leaf(__node_0),
            lparse::Node::Leaf(__node_1),
            lparse::Node::Leaf(__node_2),
            lparse::Node::Leaf(__node_3),
            lparse::Node::Leaf(__node_4),
            ],
        ) if true && __node_0.token_type == LParseToken::LSquareBracket
            && __node_1.token_type == LParseToken::Ident
            && __node_2.token_type == LParseToken::Colon
            && __node_3.token_type == LParseToken::Ident
            && __node_4.token_type == LParseToken::RSquareBracket => {
            let _ = __node_0.lexeme;
            let bind_to = __node_1.lexeme;
            let _ = __node_2.lexeme;
            let token = __node_3.lexeme;
            let _ = __node_4.lexeme;
            BoundLeaf {
                token: token,
                bind_to: bind_to,
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_bound_rule(node: &lparse::Parent<ParseRule>) -> BoundRule {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::BoundRule,
            [lparse::Node::Leaf(__node_0),
            lparse::Node::Leaf(__node_1),
            lparse::Node::Leaf(__node_2),
            lparse::Node::Leaf(__node_3),
            lparse::Node::Leaf(__node_4),
            ],
        ) if true && __node_0.token_type == LParseToken::LAngleBracket
            && __node_1.token_type == LParseToken::Ident
            && __node_2.token_type == LParseToken::Colon
            && __node_3.token_type == LParseToken::Ident
            && __node_4.token_type == LParseToken::RAngleBracket => {
            let _ = __node_0.lexeme;
            let bind_to = __node_1.lexeme;
            let _ = __node_2.lexeme;
            let token = __node_3.lexeme;
            let _ = __node_4.lexeme;
            BoundRule {
                rule: token,
                bind_to: bind_to,
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_node(node: &lparse::Parent<ParseRule>) -> LNode {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::Node,
            [lparse::Node::Parent(__node_0),
            ],
        ) if true && __node_0.rule == ParseRule::BoundLeaf => {
            let leaf = __rule_factory_function_bound_leaf(__node_0);
            LNode::Leaf(leaf)
        }
        (
            ParseRule::Node,
            [lparse::Node::Parent(__node_0),
            ],
        ) if true && __node_0.rule == ParseRule::BoundRule => {
            let rule = __rule_factory_function_bound_rule(__node_0);
            LNode::Rule(rule)
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_nodes(node: &lparse::Parent<ParseRule>) -> Vec<LNode> {
    match (&node.rule, node.children.as_slice()) {
        (ParseRule::Nodes, []) if true => Vec::new(),
        (
            ParseRule::Nodes,
            [lparse::Node::Parent(__node_0),
            lparse::Node::Parent(__node_1),
            ],
        ) if true && __node_0.rule == ParseRule::Nodes
            && __node_1.rule == ParseRule::Node => {
            let head = __rule_factory_function_nodes(__node_0);
            let node = __rule_factory_function_node(__node_1);
            {
                let mut all = head;
                all.push(node);
                all
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_production(
    node: &lparse::Parent<ParseRule>,
) -> ProductionDefinition {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::Production,
            [lparse::Node::Parent(__node_0),
            lparse::Node::Leaf(__node_1),
            lparse::Node::Leaf(__node_2),
            lparse::Node::Leaf(__node_3),
            lparse::Node::Leaf(__node_4),
            ],
        ) if true && __node_0.rule == ParseRule::Nodes
            && __node_1.token_type == LParseToken::Equals
            && __node_2.token_type == LParseToken::RAngleBracket
            && __node_3.token_type == LParseToken::EmbeddedRust
            && __node_4.token_type == LParseToken::Comma => {
            let definition = __rule_factory_function_nodes(__node_0);
            let _ = __node_1.lexeme;
            let _ = __node_2.lexeme;
            let semantic_action = __node_3.lexeme;
            let _ = __node_4.lexeme;
            ProductionDefinition {
                definition: definition,
                semantic_action: semantic_action,
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_productions(
    node: &lparse::Parent<ParseRule>,
) -> Vec<ProductionDefinition> {
    match (&node.rule, node.children.as_slice()) {
        (ParseRule::Productions, []) if true => Vec::new(),
        (
            ParseRule::Productions,
            [lparse::Node::Parent(__node_0),
            lparse::Node::Parent(__node_1),
            ],
        ) if true && __node_0.rule == ParseRule::Productions
            && __node_1.rule == ParseRule::Production => {
            let head = __rule_factory_function_productions(__node_0);
            let production = __rule_factory_function_production(__node_1);
            {
                let mut all = head;
                all.push(production);
                all
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_rule(node: &lparse::Parent<ParseRule>) -> LRule {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::Rule,
            [lparse::Node::Leaf(__node_0),
            lparse::Node::Leaf(__node_1),
            lparse::Node::Leaf(__node_2),
            lparse::Node::Leaf(__node_3),
            lparse::Node::Parent(__node_4),
            lparse::Node::Leaf(__node_5),
            lparse::Node::Leaf(__node_6),
            ],
        ) if true && __node_0.token_type == LParseToken::Ident
            && __node_1.token_type == LParseToken::Colon
            && __node_2.token_type == LParseToken::EmbeddedRust
            && __node_3.token_type == LParseToken::LCurlyBrace
            && __node_4.rule == ParseRule::Productions
            && __node_5.token_type == LParseToken::RCurlyBrace
            && __node_6.token_type == LParseToken::Semicolon => {
            let name = __node_0.lexeme;
            let _ = __node_1.lexeme;
            let return_type = __node_2.lexeme;
            let _ = __node_3.lexeme;
            let productions = __rule_factory_function_productions(__node_4);
            let _ = __node_5.lexeme;
            let _ = __node_6.lexeme;
            LRule {
                name: name,
                return_type: return_type,
                productions,
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_rules(node: &lparse::Parent<ParseRule>) -> Vec<LRule> {
    match (&node.rule, node.children.as_slice()) {
        (ParseRule::Rules, []) if true => Vec::new(),
        (
            ParseRule::Rules,
            [lparse::Node::Parent(__node_0),
            lparse::Node::Parent(__node_1),
            ],
        ) if true && __node_0.rule == ParseRule::Rules
            && __node_1.rule == ParseRule::Rule => {
            let head = __rule_factory_function_rules(__node_0);
            let rule = __rule_factory_function_rule(__node_1);
            {
                let mut all = head;
                all.push(rule);
                all
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_set_goal_rule(node: &lparse::Parent<ParseRule>) -> Ident {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::SetGoalRule,
            [lparse::Node::Leaf(__node_0),
            lparse::Node::Leaf(__node_1),
            lparse::Node::Leaf(__node_2),
            lparse::Node::Leaf(__node_3),
            ],
        ) if true && __node_0.token_type == LParseToken::GoalRule
            && __node_1.token_type == LParseToken::Equals
            && __node_2.token_type == LParseToken::Ident
            && __node_3.token_type == LParseToken::Semicolon => {
            let _ = __node_0.lexeme;
            let _ = __node_1.lexeme;
            let goalrule = __node_2.lexeme;
            let _ = __node_3.lexeme;
            goalrule
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_set_token_type(node: &lparse::Parent<ParseRule>) -> Ident {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::SetTokenType,
            [lparse::Node::Leaf(__node_0),
            lparse::Node::Leaf(__node_1),
            lparse::Node::Leaf(__node_2),
            lparse::Node::Leaf(__node_3),
            ],
        ) if true && __node_0.token_type == LParseToken::TokenType
            && __node_1.token_type == LParseToken::Equals
            && __node_2.token_type == LParseToken::Ident
            && __node_3.token_type == LParseToken::Semicolon => {
            let _ = __node_0.lexeme;
            let _ = __node_1.lexeme;
            let tokentype = __node_2.lexeme;
            let _ = __node_3.lexeme;
            tokentype
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_preamble(node: &lparse::Parent<ParseRule>) -> Vec<Ident> {
    match (&node.rule, node.children.as_slice()) {
        (ParseRule::Preamble, []) if true => Vec::new(),
        (
            ParseRule::Preamble,
            [lparse::Node::Parent(__node_0),
            lparse::Node::Leaf(__node_1),
            ],
        ) if true && __node_0.rule == ParseRule::Preamble
            && __node_1.token_type == LParseToken::RustImport => {
            let head = __rule_factory_function_preamble(__node_0);
            let import = __node_1.lexeme;
            {
                let mut all = head;
                all.push(import);
                all
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_grammar(node: &lparse::Parent<ParseRule>) -> LGrammar {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::Grammar,
            [lparse::Node::Parent(__node_0),
            lparse::Node::Parent(__node_1),
            lparse::Node::Parent(__node_2),
            lparse::Node::Parent(__node_3),
            ],
        ) if true && __node_0.rule == ParseRule::Preamble
            && __node_1.rule == ParseRule::SetGoalRule
            && __node_2.rule == ParseRule::SetTokenType
            && __node_3.rule == ParseRule::Rules => {
            let preamble = __rule_factory_function_preamble(__node_0);
            let goalrule = __rule_factory_function_set_goal_rule(__node_1);
            let tokentype = __rule_factory_function_set_token_type(__node_2);
            let rules = __rule_factory_function_rules(__node_3);
            LGrammar {
                preamble,
                goal_rule: goalrule,
                token_type: tokentype,
                rules,
            }
        }
        _ => panic!("Unreachable"),
    }
}
