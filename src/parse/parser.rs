use super::action::{Action, make_action};
use super::error::{Error, Result};
use super::goto::make_goto;
use super::grammar::*;
use super::rule::Rule;
use super::state::{State, StateId};
use crate::core::interner::Interner;
use crate::core::ordinal::Ordinal;
use crate::lexer::{Token, TokenType};
use crate::ordinal_enum;
use ndarray::Array2;
use nonempty::{NonEmpty, nonempty};
use std::collections::VecDeque;

struct Parser<R: Rule> {
    grammar: Grammar<R>,

    state_table: Interner<StateId, State>,
    goto_table: Array2<Option<StateId>>,
    action_table: Array2<Action>,
    initial_state_id: StateId,
}

type Stack<R: Rule> = Vec<(StateId, Node<R>)>;

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
        let mut curr_state_id = self.initial_state_id;
        let mut stack = Stack::<R>::new();

        let mut iter = token_stream.peekable();

        loop {
            let current_state = self.state_table.get_left(curr_state_id);
            println!("[PARSE TIME] Currently at {current_state:?} with {curr_state_id:?}");
            let next_token = iter.peek().ok_or(Error::IncompleteProgram)?;
            let produced_node: Node<R> =
                match self.action_table[[curr_state_id.0, next_token.token_type.ord()]] {
                    Action::Shift => {
                        println!("[PARSE TIME] shifting");

                        match iter.next() {
                            Some(token) => Node::Terminal(token),
                            None => return Err(Error::ExpectedToken),
                        }
                    }
                    Action::Reduce(production_id) => {
                        println!("[PARSE TIME] reducing");
                        let production = self.grammar.production(production_id);

                        match stack.len().checked_sub(production.len()) {
                            Some(drain_from) => {
                                curr_state_id = stack[drain_from].0;
                                println!("[PARSE TIME] returning to {curr_state_id:?}");
                                let drained: Vec<_> =
                                    stack.drain(drain_from..).map(|(_, node)| node).collect();

                                let make_rule = &production.make_rule;
                                Node::NonTerminal(make_rule(drained))
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
                            Some((_, Node::NonTerminal(rule)))
                                if rule.get_type() == self.grammar.target_rule() =>
                            {
                                Ok(rule)
                            }
                            _ => Err(Error::IncompleteProgram),
                        };
                    }
                    Action::Abort => return Err(Error::IncompleteProgram),
                };

            let prev_state = curr_state_id;

            match self.goto_table[[curr_state_id.0, produced_node.symbol().ord()]] {
                Some(next_state_id) => {
                    curr_state_id = next_state_id;
                }
                None => {
                    let sym = produced_node.symbol();
                    println!("Unrecognized token {sym:?}");
                    return Err(Error::UnrecognizedToken);
                }
            }

            println!("[PARSE TIME] push into stack ({prev_state:?},{produced_node:?})");
            stack.push((prev_state, produced_node));
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::frontend::token::{LoxToken, LoxTokenType, lex};
    use lasso::{Rodeo, Spur};

    ordinal_enum!(TestRuleType {
        Alpha,
        Beta,
        Plus,
        Times,
        Literal,
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
        Plus(Box<TestRule>, Box<TestRule>),
        Times(Box<TestRule>, Box<TestRule>),
        Literal(LoxToken),
    }

    impl Rule for TestRule {
        type RuleType = TestRuleType;
        type TokenType = LoxTokenType;

        fn get_type(&self) -> Self::RuleType {
            match self {
                TestRule::Alpha | TestRule::Group(_) => TestRuleType::Alpha,
                TestRule::Beta(_, _) => TestRuleType::Beta,
                TestRule::Plus(_, _) => TestRuleType::Plus,
                TestRule::Times(_, _) => TestRuleType::Times,
                TestRule::Literal(_) => TestRuleType::Literal,
            }
        }
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
            make_rule: Box::new(move |nodes| {
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
                            && rt.token_type == LoxTokenType::RParen
                            && beta.get_type() == TestRuleType::Beta =>
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
                    (Node::NonTerminal(a), Node::NonTerminal(b))
                        if a.get_type() == TestRuleType::Alpha
                            && b.get_type() == TestRuleType::Alpha =>
                    {
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
    /// expr := sum
    /// sum := term '+' sum | term;
    /// term := paren '*' term | paren;
    /// paren := literal | '(' expr ')';
    /// literal := 'num';
    ///
    ///
    fn expression_grammar() -> Grammar<TestRule> {
        todo!("expression grammar");
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
}
