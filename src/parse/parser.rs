use super::action::{Action, make_action};
use super::debug::DisplayWithGrammarExt;
use super::error::{Error, Result};
use super::goto::make_goto;
use super::grammar::*;
use super::rule::Rule;
use super::state::{State, StateId};
use crate::core::Ordinal;
use crate::core::interner::Interner;
use crate::lexer::{Token, Tokens};
use lasso::Rodeo;
use ndarray::Array2;

#[derive(Debug)]
pub struct Tree<R: Rule> {
    pub lexeme_arena: Rodeo,
    pub root: Node<R>,
}

#[derive(Debug)]
pub struct Parent<R: Rule> {
    pub rule: R,
    pub children: Vec<Node<R>>,
}

#[derive(Debug)]
pub enum Node<R: Rule> {
    Leaf(Token<R::TokenType>),
    Parent(Parent<R>),
}

impl<R: Rule> Node<R> {
    pub fn symbol(&self) -> Symbol<R> {
        match self {
            Self::Leaf(token) => Symbol::Token(token.token_type),
            Self::Parent(Parent { rule, children: _ }) => Symbol::Rule(*rule),
        }
    }
}

pub struct Parser<R: Rule> {
    grammar: Grammar<R>,

    state_table: Interner<StateId, State>,
    goto_table: Array2<Option<StateId>>,
    action_table: Array2<Action>,
    initial_state_id: StateId,
}

type Stack<R> = Vec<(StateId, Node<R>)>;

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

    pub fn parse(&self, tokens: Tokens<R::TokenType>) -> Result<Tree<R>> {
        let mut curr_state_id = self.initial_state_id;
        let mut stack = Stack::<R>::new();

        let mut iter = tokens.iter().peekable();

        loop {
            let next_token = iter.peek().ok_or(Error::IncompleteProgram)?;

            // We want to support rules that return nodes that are _not_ of the type of that rule.
            let action = &self.action_table[[curr_state_id.0, next_token.token_type.ord()]];

            let next_node: Node<R> = match action {
                Action::Shift => match iter.next() {
                    Some(token) => Node::Leaf(*token),
                    None => return Err(Error::ExpectedToken),
                },
                Action::Reduce(production_id) | Action::Accept(production_id) => {
                    let production = self.grammar.production(*production_id);

                    match stack.len().checked_sub(production.len()) {
                        Some(drain_from) => {
                            curr_state_id = stack[drain_from].0;

                            Node::Parent(Parent {
                                rule: production.rule,
                                children: stack.drain(drain_from..).map(|(_, node)| node).collect(),
                            })
                        }
                        None => {
                            return Err(Error::IncompleteProgram);
                        }
                    }
                }
            };

            if let Action::Accept(_) = action {
                if !stack.is_empty() {
                    return Err(Error::ExcessProgram);
                }

                match next_node {
                    Node::Parent(p) if p.rule == self.grammar.target_rule() => {
                        return Ok(Tree {
                            lexeme_arena: tokens.lexeme_arena,
                            root: Node::Parent(p),
                        });
                    }
                    _ => panic!("unreachable"),
                }
            }

            let prev_state = curr_state_id;

            match self.goto_table[[curr_state_id.0, next_node.symbol().ord()]] {
                Some(next_state_id) => {
                    curr_state_id = next_state_id;
                }
                None => {
                    return Err(Error::UnrecognizedToken);
                }
            }

            stack.push((prev_state, next_node));
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::frontend::token::{LoxLexer, LoxTokenKind};
    use lasso::Rodeo;
    use lox_derive::Ordinal;
    use nonempty::nonempty;
    use strum::Display;

    // Grammar (BNF, start symbol = <Beta>):
    //   <Beta>  ::= <Alpha> <Alpha>
    //   <Alpha> ::= "unit"
    //             | "(" <Beta> ")"
    #[derive(Ordinal, Eq, PartialEq, Debug, Display, PartialOrd, Ord, Hash)]
    enum TestRule {
        Plus,
        Times,
        Literal,
        Expr,
        Paren,
    }

    impl Rule for TestRule {
        type TokenType = LoxTokenKind;

        fn goal() -> Self {
            TestRule::Expr
        }
    }

    type TestProduction = Production<TestRule>;

    type ExprNode = Node<TestRule>;
    type TestTree = Tree<TestRule>;

    enum Expression {
        Sum(Box<Expression>, Box<Expression>),
        Times(Box<Expression>, Box<Expression>),
        Literal(i64),
    }

    impl Expression {
        pub fn from_cst(root: &TestTree, node: &ExprNode) -> Self {
            match node {
                ExprNode::Leaf(_token) => panic!("Tokens cannot be directly parsed as expr"),
                ExprNode::Parent(Parent { rule, children }) => match (rule, children.as_slice()) {
                    (TestRule::Plus, [lhs, ExprNode::Leaf(_plus), rhs])
                        if _plus.token_type == LoxTokenKind::Plus =>
                    {
                        Expression::Sum(
                            Box::new(Self::from_cst(root, lhs)),
                            Box::new(Self::from_cst(root, rhs)),
                        )
                    }
                    (TestRule::Literal, [ExprNode::Leaf(literal)]) => Expression::Literal(
                        root.lexeme_arena.resolve(&literal.lexeme).parse().unwrap(),
                    ),
                    (TestRule::Times, [lhs, ExprNode::Leaf(_times), rhs])
                        if _times.token_type == LoxTokenKind::Star =>
                    {
                        Expression::Times(
                            Box::new(Self::from_cst(root, lhs)),
                            Box::new(Self::from_cst(root, rhs)),
                        )
                    }
                    (TestRule::Paren, [ExprNode::Leaf(lparen), x, ExprNode::Leaf(rparen)])
                        if lparen.token_type == LoxTokenKind::LParen
                            && rparen.token_type == LoxTokenKind::RParen =>
                    {
                        Self::from_cst(root, x)
                    }
                    // fallthrough
                    (TestRule::Expr | TestRule::Paren | TestRule::Plus | TestRule::Times, [x]) => {
                        Self::from_cst(root, x)
                    }
                    _ => {
                        panic!("unreachable state: {:?}", node)
                    }
                },
            }
        }

        pub fn eval(&self) -> i64 {
            match self {
                Expression::Sum(lhs, rhs) => lhs.eval() + rhs.eval(),
                Expression::Times(lhs, rhs) => lhs.eval() * rhs.eval(),
                &Expression::Literal(x) => x,
            }
        }
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
            rule: TestRule::Expr,
            definition: nonempty![Symbol::Rule(TestRule::Plus)],
        };

        let sum_recursive_def = TestProduction {
            rule: TestRule::Plus,
            definition: nonempty![
                Symbol::Rule(TestRule::Times),
                Symbol::Token(LoxTokenKind::Plus),
                Symbol::Rule(TestRule::Plus),
            ],
        };

        let sum_fallthrough_def = TestProduction {
            rule: TestRule::Plus,
            definition: nonempty![Symbol::Rule(TestRule::Times),],
        };

        let term_recursive_def = TestProduction {
            rule: TestRule::Times,
            definition: nonempty![
                Symbol::Rule(TestRule::Paren),
                Symbol::Token(LoxTokenKind::Star),
                Symbol::Rule(TestRule::Times),
            ],
        };

        let term_fallthrough_def = TestProduction {
            rule: TestRule::Times,
            definition: nonempty![Symbol::Rule(TestRule::Paren),],
        };

        let paren_recursive_def = TestProduction {
            rule: TestRule::Paren,
            definition: nonempty![
                Symbol::Token(LoxTokenKind::LParen),
                Symbol::Rule(TestRule::Expr),
                Symbol::Token(LoxTokenKind::RParen),
            ],
        };

        let paren_fallthrough_def = TestProduction {
            rule: TestRule::Paren,
            definition: nonempty![Symbol::Rule(TestRule::Literal),],
        };

        let literal_def = TestProduction {
            rule: TestRule::Literal,
            definition: nonempty![Symbol::Token(LoxTokenKind::Number)],
        };

        Grammar::new(vec![
            expr_def,
            sum_recursive_def,
            sum_fallthrough_def,
            term_recursive_def,
            term_fallthrough_def,
            paren_recursive_def,
            paren_fallthrough_def,
            literal_def,
        ])
    }

    struct ExprFixture {
        rodeo: Rodeo,
        lexer: LoxLexer,
        parser: Parser<TestRule>,
    }

    impl ExprFixture {
        fn new() -> Self {
            let rodeo = Rodeo::default();
            let lexer = LoxLexer::new().unwrap();
            Self {
                rodeo,
                lexer,
                parser: Parser::new(expression_grammar()),
            }
        }

        fn eval(&mut self, input: &str) -> i64 {
            let tokens = self.lexer.lex(input).unwrap();
            let cst = self.parser.parse(tokens).unwrap();
            Expression::from_cst(&cst, &cst.root).eval()
        }
    }

    #[test]
    fn expr_test() {
        let mut f = ExprFixture::new();
        assert_eq!(f.eval("1"), 1);
        assert_eq!(f.eval("1+2"), 3);
        assert_eq!(f.eval("((((((((1))))))))"), 1);
        assert_eq!(f.eval(r#"((((((((((1)))))))*((((((2)))))))))"#), 2);
        assert_eq!(f.eval("1*2+3"), 5);
        assert_eq!(
            f.eval("(1+2*3)*(4+(5*6+7))+8*(9+10*11*(12+13)+14)+15"),
            22486
        );
        assert_eq!(
            f.eval("1+2+3*4*5+(6+7)*(8+9*1)+10*11*12+13+(14+15*16)*(17+18)+19+20"),
            10546
        );
        assert_eq!(
            f.eval("((((1+2)*(3+4))+((5*6)+(7*8)))*((9+10)*(11+12)))+((13*14+15)*(16+17*18))"),
            110193
        );
        assert_eq!(
            f.eval("(((((1+2+3))*((4*5*6))+((7+8)*(9+10+11)))))+12*((13+14))*(15+16)+17+18"),
            11249
        );
    }
}
