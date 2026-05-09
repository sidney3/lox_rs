use super::action::{Action, make_action};
use super::error::{Error, Result};
use super::goto::make_goto;
use super::grammar::*;
use super::rule::Rule;
use super::state::{State, StateId};
use crate::core::interner::Interner;
use crate::core::ordinal::Ordinal;
use crate::lexer::Token;
use crate::ordinal_enum;
use ndarray::Array2;
use nonempty::nonempty;

struct Parser<R: Rule> {
    grammar: Grammar<R>,

    state_table: Interner<StateId, State>,
    goto_table: Array2<Option<StateId>>,
    action_table: Array2<Action>,
    initial_state_id: StateId,
}

type Stack<R> = Vec<(StateId, Symbol<R>, Node<R>)>;

impl<R: Rule> Parser<R> {
    pub fn new(grammar: Grammar<R>) -> Self {
        let mut state_table = Interner::new();

        let goto_table = make_goto(&grammar, &mut state_table);
        let action_table = make_action(&grammar, &state_table);
        let initial_state_id = state_table.intern(State::initial(&grammar));

        Self {
            grammar,
            initial_state_id,
            state_table,
            goto_table,
            action_table,
        }
    }

    pub fn parse(&self, token_stream: impl Iterator<Item = Token<R::TokenType>>) -> Result<R> {
        println!("------ START PARSE ------");
        let mut curr_state_id = self.initial_state_id;
        let mut stack = Stack::<R>::new();

        let mut iter = token_stream.peekable();

        loop {
            let current_state = self.state_table.get_left(curr_state_id);
            println!("[PARSE TIME] Currently at {current_state:?} with {curr_state_id:?}");
            let next_token = iter.peek().ok_or(Error::IncompleteProgram)?;

            // We want to support rules that return nodes that are _not_ of the type of that rule.
            let (next_symbol, next_node): (Symbol<R>, Node<R>) =
                match self.action_table[[curr_state_id.0, next_token.token_type.ord()]] {
                    Action::Shift => {
                        println!("[PARSE TIME] shifting");

                        match iter.next() {
                            Some(token) => (Symbol::Token(token.token_type), Node::Terminal(token)),
                            None => return Err(Error::ExpectedToken),
                        }
                    }
                    Action::Reduce(production_id) => {
                        let production = self.grammar.production(production_id);
                        let reducing_to_rule = production.rule;
                        println!("[PARSE TIME] reducing to {reducing_to_rule:?}");

                        match stack.len().checked_sub(production.len()) {
                            Some(drain_from) => {
                                curr_state_id = stack[drain_from].0;
                                println!("[PARSE TIME] returning to {curr_state_id:?}");
                                let drained: Vec<_> =
                                    stack.drain(drain_from..).map(|(_, _, node)| node).collect();

                                let make_rule = &production.make_rule;
                                (
                                    Symbol::Rule(production.rule),
                                    Node::NonTerminal(make_rule(drained)),
                                )
                            }
                            None => {
                                return Err(Error::StackTooSmall);
                            }
                        }
                    }

                    Action::Accept => {
                        if stack.len() > 1 {
                            return Err(Error::ExcessProgram);
                        }

                        println!("[PARSE TIME] accepting. stack is {stack:?}");

                        return match stack.pop() {
                            Some((_, Symbol::Rule(rule_type), Node::NonTerminal(rule)))
                                if rule_type == self.grammar.target_rule() =>
                            {
                                Ok(rule)
                            }
                            _ => Err(Error::IncompleteProgram),
                        };
                    }
                };

            let prev_state = curr_state_id;

            match self.goto_table[[curr_state_id.0, next_symbol.ord()]] {
                Some(next_state_id) => {
                    curr_state_id = next_state_id;
                }
                None => {
                    println!("Unrecognized token {next_symbol:?}");
                    return Err(Error::UnrecognizedToken);
                }
            }

            println!("[PARSE TIME] push into stack ({prev_state:?},{next_node:?})");
            stack.push((prev_state, next_symbol, next_node));
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::frontend::token::{LoxToken, LoxTokenType, lex};
    use lasso::Rodeo;

    ordinal_enum!(TestRuleType {
        Alpha,
        Beta,
        Plus,
        Times,
        Literal,
        Expr,
        Paren,
    });

    // Grammar (BNF, start symbol = <Beta>):
    //   <Beta>  ::= <Alpha> <Alpha>
    //   <Alpha> ::= "unit"
    //             | "(" <Beta> ")"
    #[derive(Hash, PartialEq, Eq, Clone, Debug)]
    enum TestRule {
        Alpha,
        Group(Box<TestRule>),
        Beta(Box<TestRule>, Box<TestRule>),
        Expr(Box<TestRule>),
        Plus(Box<TestRule>, Box<TestRule>),
        Times(Box<TestRule>, Box<TestRule>),
        Literal(LoxToken),
    }

    impl TestRule {
        pub fn reduce_expr(self) -> Self {
            match self {
                TestRule::Expr(_) => compact_expr(self),
                TestRule::Plus(lhs, rhs) => {
                    TestRule::Plus(Box::new(lhs.reduce_expr()), Box::new(rhs.reduce_expr()))
                }
                TestRule::Times(lhs, rhs) => {
                    TestRule::Times(Box::new(lhs.reduce_expr()), Box::new(rhs.reduce_expr()))
                }
                _ => self,
            }
        }
    }

    fn compact_expr(e: TestRule) -> TestRule {
        if let TestRule::Expr(inner) = e {
            let mut inner_e = *inner;

            while let TestRule::Expr(inner_inner_e) = inner_e {
                inner_e = *inner_inner_e;
            }

            TestRule::Expr(Box::new(inner_e))
        } else {
            e
        }
    }

    impl Rule for TestRule {
        type RuleType = TestRuleType;
        type TokenType = LoxTokenType;
    }

    type TestProduction = Production<TestRule>;

    ///
    /// alpha := 'True' | '(' beta ')'
    /// beta := alpha alpha
    ///
    /// target_rule = beta
    ///
    fn test_grammar() -> Grammar<TestRule> {
        let alpha_unit = TestProduction {
            rule: TestRuleType::Alpha,
            make_rule: Box::new(|nodes| {
                let [t1] = <[_; 1]>::try_from(nodes).unwrap();
                match t1 {
                    Node::Terminal(t) if t.token_type == LoxTokenType::True => TestRule::Alpha,
                    _ => panic!("unreachable"),
                }
            }),
            definition: nonempty![Symbol::Token(LoxTokenType::True)],
        };
        let alpha_group = TestProduction {
            rule: TestRuleType::Alpha,
            make_rule: Box::new(|nodes| {
                let [l, b, r] = <[_; 3]>::try_from(nodes).unwrap();
                match (l, b, r) {
                    (Node::Terminal(lt), Node::NonTerminal(beta), Node::Terminal(rt))
                        if lt.token_type == LoxTokenType::LParen
                            && rt.token_type == LoxTokenType::RParen =>
                    {
                        TestRule::Group(Box::new(beta))
                    }
                    _ => panic!("unreachable"),
                }
            }),
            definition: nonempty![
                Symbol::Token(LoxTokenType::LParen),
                Symbol::Rule(TestRuleType::Beta),
                Symbol::Token(LoxTokenType::RParen),
            ],
        };
        let beta_def = TestProduction {
            rule: TestRuleType::Beta,
            make_rule: Box::new(|nodes| {
                let [r1, r2] = <[_; 2]>::try_from(nodes).unwrap();
                match (r1, r2) {
                    (Node::NonTerminal(a), Node::NonTerminal(b)) => {
                        TestRule::Beta(Box::new(a), Box::new(b))
                    }
                    _ => panic!("unreachable"),
                }
            }),
            definition: nonempty![
                Symbol::Rule(TestRuleType::Alpha),
                Symbol::Rule(TestRuleType::Alpha)
            ],
        };

        Grammar::new(vec![alpha_unit, alpha_group, beta_def], TestRuleType::Beta)
    }

    ///
    ///
    /// yacc (style) definition
    ///
    /// expr : sum { TestRule::Expr(Box::new($1)) };
    ///
    /// sum  : term '+' sum { TestRule::Plus(Box::new($1), Box::new($3)) }
    ///      | term { $1 };
    ///
    /// term : paren '*' term  { TestRule::Times(Box::new($1), Box::new($3)) }
    ///      | paren { $1 };
    ///
    /// paren := '(' expr ')' { $2 }
    ///      | literal { $1 };
    ///
    /// literal := 'num' { Literal::new($1) };
    ///
    ///
    fn expression_grammar() -> Grammar<TestRule> {
        let expr_def = TestProduction {
            rule: TestRuleType::Expr,
            make_rule: Box::new(|nodes| {
                if let [Node::NonTerminal(r)] = <[_; 1]>::try_from(nodes).unwrap() {
                    TestRule::Expr(Box::new(r))
                } else {
                    panic!("unreachable")
                }
            }),
            definition: nonempty![Symbol::Rule(TestRuleType::Plus)],
        };

        let sum_recursive_def = TestProduction {
            rule: TestRuleType::Plus,
            make_rule: Box::new(|nodes| {
                if let [Node::NonTerminal(lhs), _, Node::NonTerminal(rhs)] =
                    <[_; 3]>::try_from(nodes).unwrap()
                {
                    TestRule::Plus(Box::new(lhs), Box::new(rhs))
                } else {
                    panic!("unreachable")
                }
            }),
            definition: nonempty![
                Symbol::Rule(TestRuleType::Times),
                Symbol::Token(LoxTokenType::Plus),
                Symbol::Rule(TestRuleType::Plus),
            ],
        };

        let sum_fallthrough_def = TestProduction {
            rule: TestRuleType::Plus,
            make_rule: Box::new(|nodes| {
                if let [Node::NonTerminal(r)] = <[_; 1]>::try_from(nodes).unwrap() {
                    r
                } else {
                    panic!("unreachable")
                }
            }),
            definition: nonempty![Symbol::Rule(TestRuleType::Times),],
        };

        let term_recursive_def = TestProduction {
            rule: TestRuleType::Times,
            make_rule: Box::new(|nodes| {
                if let [Node::NonTerminal(lhs), _, Node::NonTerminal(rhs)] =
                    <[_; 3]>::try_from(nodes).unwrap()
                {
                    TestRule::Times(Box::new(lhs), Box::new(rhs))
                } else {
                    panic!("unreachable")
                }
            }),
            definition: nonempty![
                Symbol::Rule(TestRuleType::Paren),
                Symbol::Token(LoxTokenType::Star),
                Symbol::Rule(TestRuleType::Times),
            ],
        };

        let term_fallthrough_def = TestProduction {
            rule: TestRuleType::Times,
            make_rule: Box::new(|nodes| {
                if let [Node::NonTerminal(r)] = <[_; 1]>::try_from(nodes).unwrap() {
                    r
                } else {
                    panic!("unreachable")
                }
            }),
            definition: nonempty![Symbol::Rule(TestRuleType::Paren),],
        };

        let paren_recursive_def = TestProduction {
            rule: TestRuleType::Paren,
            make_rule: Box::new(|nodes| {
                if let [_, Node::NonTerminal(expr), _] = <[_; 3]>::try_from(nodes).unwrap() {
                    expr
                } else {
                    panic!("unreachable")
                }
            }),
            definition: nonempty![
                Symbol::Token(LoxTokenType::LParen),
                Symbol::Rule(TestRuleType::Expr),
                Symbol::Token(LoxTokenType::RParen),
            ],
        };

        let paren_fallthrough_def = TestProduction {
            rule: TestRuleType::Paren,
            make_rule: Box::new(|nodes| {
                if let [Node::NonTerminal(r)] = <[_; 1]>::try_from(nodes).unwrap() {
                    r
                } else {
                    panic!("unreachable")
                }
            }),
            definition: nonempty![Symbol::Rule(TestRuleType::Literal),],
        };

        let literal_def = TestProduction {
            rule: TestRuleType::Literal,
            make_rule: Box::new(|nodes| {
                if let [Node::Terminal(t)] = <[_; 1]>::try_from(nodes).unwrap() {
                    TestRule::Literal(t)
                } else {
                    panic!("unreachable")
                }
            }),
            definition: nonempty![Symbol::Token(LoxTokenType::Number)],
        };

        Grammar::new(
            vec![
                expr_def,
                sum_recursive_def,
                sum_fallthrough_def,
                term_recursive_def,
                term_fallthrough_def,
                paren_recursive_def,
                paren_fallthrough_def,
                literal_def,
            ],
            TestRuleType::Expr,
        )
    }

    #[test]
    fn trivial_parsing() {
        let mut rodeo = Rodeo::default();
        let tokens = lex("true true", &mut rodeo).unwrap();
        let parser = Parser::new(test_grammar());

        assert_eq!(
            parser.parse(tokens.into_iter()).unwrap(),
            TestRule::Beta(Box::new(TestRule::Alpha), Box::new(TestRule::Alpha),)
        )
    }

    #[test]
    fn nested_parsing() {
        // Input:    ( unit unit ) unit
        // Expected: Beta(Group(Beta(Alpha, Alpha)), Alpha)
        let mut rodeo = Rodeo::default();
        let tokens = lex("( true true ) true", &mut rodeo).unwrap();
        let parser = Parser::new(test_grammar());

        assert_eq!(
            parser.parse(tokens.into_iter()).unwrap(),
            TestRule::Beta(
                Box::new(TestRule::Group(Box::new(TestRule::Beta(
                    Box::new(TestRule::Alpha),
                    Box::new(TestRule::Alpha),
                )))),
                Box::new(TestRule::Alpha),
            )
        )
    }

    #[test]
    fn expression_parsing() {
        let mut rodeo = Rodeo::default();
        let parser = Parser::new(expression_grammar());

        let one = Token {
            lexeme: rodeo.get_or_intern("1"),
            token_type: LoxTokenType::Number,
            line: 1,
        };
        let two = Token {
            lexeme: rodeo.get_or_intern("2"),
            token_type: LoxTokenType::Number,
            line: 1,
        };

        let tokens = lex("1", &mut rodeo).unwrap();
        assert_eq!(
            parser.parse(tokens.into_iter()).unwrap(),
            TestRule::Expr(Box::new(TestRule::Literal(one)))
        );
        let tokens = lex("1+2", &mut rodeo).unwrap();
        assert_eq!(
            parser.parse(tokens.into_iter()).unwrap(),
            TestRule::Expr(Box::new(TestRule::Plus(
                Box::new(TestRule::Literal(one)),
                Box::new(TestRule::Literal(two)),
            )))
        );

        let tokens = lex("((1))", &mut rodeo).unwrap();
        assert_eq!(
            parser.parse(tokens.into_iter()).unwrap().reduce_expr(),
            TestRule::Expr(Box::new(TestRule::Literal(one)))
        );
    }
}
