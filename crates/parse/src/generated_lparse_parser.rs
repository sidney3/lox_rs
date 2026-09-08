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
use std::collections::VecDeque;
use parse;
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
impl parse::Rule for ParseRule {
    type TokenType = LParseToken;
}
pub struct LParseParser {
    parser: parse::Parser<ParseRule>,
}
impl LParseParser {
    pub fn new() -> Self {
        Self {
            parser: parse::Parser::new(__make_grammar()),
        }
    }
    pub fn parse(
        &self,
        tokens: Tokens<LParseToken>,
    ) -> Result<(Rodeo, LGrammar), parse::Error> {
        let cst = self.parser.parse(tokens)?;
        if let parse::Node::Parent(root) = &cst.root {
            Ok((cst.lexeme_arena, __rule_factory_function_grammar(&root)))
        } else {
            panic!("Unreachable, root of CST is a token");
        }
    }
}
fn __make_grammar() -> parse::Grammar<ParseRule> {
    parse::Grammar::new(
        ParseRule::Grammar,
        Vec::from([
            parse::Production::<ParseRule> {
                rule: ParseRule::BoundLeaf,
                definition: Vec::from([
                    parse::Symbol::Token(LParseToken::LSquareBracket),
                    parse::Symbol::Token(LParseToken::Ident),
                    parse::Symbol::Token(LParseToken::Colon),
                    parse::Symbol::Token(LParseToken::Ident),
                    parse::Symbol::Token(LParseToken::RSquareBracket),
                ]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::BoundRule,
                definition: Vec::from([
                    parse::Symbol::Token(LParseToken::LAngleBracket),
                    parse::Symbol::Token(LParseToken::Ident),
                    parse::Symbol::Token(LParseToken::Colon),
                    parse::Symbol::Token(LParseToken::Ident),
                    parse::Symbol::Token(LParseToken::RAngleBracket),
                ]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Node,
                definition: Vec::from([parse::Symbol::Rule(ParseRule::BoundLeaf)]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Node,
                definition: Vec::from([parse::Symbol::Rule(ParseRule::BoundRule)]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Nodes,
                definition: Vec::from([]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Nodes,
                definition: Vec::from([
                    parse::Symbol::Rule(ParseRule::Node),
                    parse::Symbol::Rule(ParseRule::Nodes),
                ]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Production,
                definition: Vec::from([
                    parse::Symbol::Rule(ParseRule::Nodes),
                    parse::Symbol::Token(LParseToken::Equals),
                    parse::Symbol::Token(LParseToken::RAngleBracket),
                    parse::Symbol::Token(LParseToken::EmbeddedRust),
                    parse::Symbol::Token(LParseToken::Comma),
                ]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Productions,
                definition: Vec::from([]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Productions,
                definition: Vec::from([
                    parse::Symbol::Rule(ParseRule::Production),
                    parse::Symbol::Rule(ParseRule::Productions),
                ]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Rule,
                definition: Vec::from([
                    parse::Symbol::Token(LParseToken::Ident),
                    parse::Symbol::Token(LParseToken::Colon),
                    parse::Symbol::Token(LParseToken::EmbeddedRust),
                    parse::Symbol::Token(LParseToken::LCurlyBrace),
                    parse::Symbol::Rule(ParseRule::Productions),
                    parse::Symbol::Token(LParseToken::RCurlyBrace),
                    parse::Symbol::Token(LParseToken::Semicolon),
                ]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Rules,
                definition: Vec::from([]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Rules,
                definition: Vec::from([
                    parse::Symbol::Rule(ParseRule::Rule),
                    parse::Symbol::Rule(ParseRule::Rules),
                ]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::SetGoalRule,
                definition: Vec::from([
                    parse::Symbol::Token(LParseToken::GoalRule),
                    parse::Symbol::Token(LParseToken::Equals),
                    parse::Symbol::Token(LParseToken::Ident),
                    parse::Symbol::Token(LParseToken::Semicolon),
                ]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::SetTokenType,
                definition: Vec::from([
                    parse::Symbol::Token(LParseToken::TokenType),
                    parse::Symbol::Token(LParseToken::Equals),
                    parse::Symbol::Token(LParseToken::Ident),
                    parse::Symbol::Token(LParseToken::Semicolon),
                ]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Preamble,
                definition: Vec::from([]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Preamble,
                definition: Vec::from([
                    parse::Symbol::Token(LParseToken::RustImport),
                    parse::Symbol::Rule(ParseRule::Preamble),
                ]),
            },
            parse::Production::<ParseRule> {
                rule: ParseRule::Grammar,
                definition: Vec::from([
                    parse::Symbol::Rule(ParseRule::Preamble),
                    parse::Symbol::Rule(ParseRule::SetGoalRule),
                    parse::Symbol::Rule(ParseRule::SetTokenType),
                    parse::Symbol::Rule(ParseRule::Rules),
                ]),
            },
        ]),
    )
}
fn __rule_factory_function_bound_leaf(node: &parse::Parent<ParseRule>) -> BoundLeaf {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::BoundLeaf,
            [parse::Node::Leaf(__node_0),
            parse::Node::Leaf(__node_1),
            parse::Node::Leaf(__node_2),
            parse::Node::Leaf(__node_3),
            parse::Node::Leaf(__node_4),
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
fn __rule_factory_function_bound_rule(node: &parse::Parent<ParseRule>) -> BoundRule {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::BoundRule,
            [parse::Node::Leaf(__node_0),
            parse::Node::Leaf(__node_1),
            parse::Node::Leaf(__node_2),
            parse::Node::Leaf(__node_3),
            parse::Node::Leaf(__node_4),
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
fn __rule_factory_function_node(node: &parse::Parent<ParseRule>) -> LNode {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::Node,
            [parse::Node::Parent(__node_0),
            ],
        ) if true && __node_0.rule == ParseRule::BoundLeaf => {
            let leaf = __rule_factory_function_bound_leaf(__node_0);
            LNode::Leaf(leaf)
        }
        (
            ParseRule::Node,
            [parse::Node::Parent(__node_0),
            ],
        ) if true && __node_0.rule == ParseRule::BoundRule => {
            let rule = __rule_factory_function_bound_rule(__node_0);
            LNode::Rule(rule)
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_nodes(node: &parse::Parent<ParseRule>) -> VecDeque<LNode> {
    match (&node.rule, node.children.as_slice()) {
        (ParseRule::Nodes, []) if true => VecDeque::new(),
        (
            ParseRule::Nodes,
            [parse::Node::Parent(__node_0),
            parse::Node::Parent(__node_1),
            ],
        ) if true && __node_0.rule == ParseRule::Node
            && __node_1.rule == ParseRule::Nodes => {
            let head = __rule_factory_function_node(__node_0);
            let tail = __rule_factory_function_nodes(__node_1);
            {
                let mut all = tail;
                all.push_front(head);
                all
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_production(
    node: &parse::Parent<ParseRule>,
) -> ProductionDefinition {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::Production,
            [parse::Node::Parent(__node_0),
            parse::Node::Leaf(__node_1),
            parse::Node::Leaf(__node_2),
            parse::Node::Leaf(__node_3),
            parse::Node::Leaf(__node_4),
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
    node: &parse::Parent<ParseRule>,
) -> VecDeque<ProductionDefinition> {
    match (&node.rule, node.children.as_slice()) {
        (ParseRule::Productions, []) if true => VecDeque::new(),
        (
            ParseRule::Productions,
            [parse::Node::Parent(__node_0),
            parse::Node::Parent(__node_1),
            ],
        ) if true && __node_0.rule == ParseRule::Production
            && __node_1.rule == ParseRule::Productions => {
            let head = __rule_factory_function_production(__node_0);
            let tail = __rule_factory_function_productions(__node_1);
            {
                let mut all = tail;
                all.push_front(head);
                all
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_rule(node: &parse::Parent<ParseRule>) -> LRule {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::Rule,
            [parse::Node::Leaf(__node_0),
            parse::Node::Leaf(__node_1),
            parse::Node::Leaf(__node_2),
            parse::Node::Leaf(__node_3),
            parse::Node::Parent(__node_4),
            parse::Node::Leaf(__node_5),
            parse::Node::Leaf(__node_6),
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
fn __rule_factory_function_rules(node: &parse::Parent<ParseRule>) -> VecDeque<LRule> {
    match (&node.rule, node.children.as_slice()) {
        (ParseRule::Rules, []) if true => VecDeque::new(),
        (
            ParseRule::Rules,
            [parse::Node::Parent(__node_0),
            parse::Node::Parent(__node_1),
            ],
        ) if true && __node_0.rule == ParseRule::Rule
            && __node_1.rule == ParseRule::Rules => {
            let head = __rule_factory_function_rule(__node_0);
            let tail = __rule_factory_function_rules(__node_1);
            {
                let mut all = tail;
                all.push_front(head);
                all
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_set_goal_rule(node: &parse::Parent<ParseRule>) -> Ident {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::SetGoalRule,
            [parse::Node::Leaf(__node_0),
            parse::Node::Leaf(__node_1),
            parse::Node::Leaf(__node_2),
            parse::Node::Leaf(__node_3),
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
fn __rule_factory_function_set_token_type(node: &parse::Parent<ParseRule>) -> Ident {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::SetTokenType,
            [parse::Node::Leaf(__node_0),
            parse::Node::Leaf(__node_1),
            parse::Node::Leaf(__node_2),
            parse::Node::Leaf(__node_3),
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
fn __rule_factory_function_preamble(node: &parse::Parent<ParseRule>) -> VecDeque<Ident> {
    match (&node.rule, node.children.as_slice()) {
        (ParseRule::Preamble, []) if true => VecDeque::new(),
        (
            ParseRule::Preamble,
            [parse::Node::Leaf(__node_0),
            parse::Node::Parent(__node_1),
            ],
        ) if true && __node_0.token_type == LParseToken::RustImport
            && __node_1.rule == ParseRule::Preamble => {
            let import = __node_0.lexeme;
            let tail = __rule_factory_function_preamble(__node_1);
            {
                let mut all = tail;
                all.push_front(import);
                all
            }
        }
        _ => panic!("Unreachable"),
    }
}
fn __rule_factory_function_grammar(node: &parse::Parent<ParseRule>) -> LGrammar {
    match (&node.rule, node.children.as_slice()) {
        (
            ParseRule::Grammar,
            [parse::Node::Parent(__node_0),
            parse::Node::Parent(__node_1),
            parse::Node::Parent(__node_2),
            parse::Node::Parent(__node_3),
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
