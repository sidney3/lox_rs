use lasso::Rodeo;
use log::debug;
use lox_core::{InternPool, Ordinal};
use ndarray::Array2;

use super::action::{Action, make_action};
use super::debug::DisplayWithGrammarExt;
use super::error::{Error, Result};
use super::goto::make_goto;
use super::grammar::*;
use super::rule::Rule;
use super::state::{State, StateId};
use lexer::{Token, Tokens};

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

  state_table: InternPool<StateId, State>,
  goto_table: Array2<Option<StateId>>,
  action_table: Array2<Action>,
  initial_state_id: StateId,
}

type Stack<R> = Vec<(StateId, Node<R>)>;

impl<R: Rule> Parser<R> {
  pub fn new(grammar: Grammar<R>) -> Self {
    let mut state_table = InternPool::new();

    let goto_table = make_goto(&grammar, &mut state_table);
    let action_table = make_action(&grammar, &state_table);
    let initial_state_id = state_table.get_or_intern(State::initial(&grammar));

    Self {
      grammar,
      initial_state_id,
      state_table,
      goto_table,
      action_table,
    }
  }

  pub fn parse(&self, tokens: Tokens<R::TokenType>) -> Result<Tree<R>> {
    debug!("start parse");
    let mut curr_state_id = self.initial_state_id;
    let mut stack = Stack::<R>::new();

    let mut iter = tokens.iter().peekable();

    loop {
      let next_token = iter.peek().ok_or(Error::IncompleteProgram).cloned()?;

      debug!(
        "Current state: {}",
        self.state_table[curr_state_id].with(&self.grammar)
      );

      // We want to support rules that return nodes that are _not_ of the type of that rule.
      let action = &self.action_table[[curr_state_id.0, next_token.token_type.ord()]];

      let next_node: Node<R> = match action {
        Action::Shift => match iter.next() {
          Some(token) => {
            debug!(
              "shift token {:?} with lexeme \"{}\"",
              token.token_type,
              tokens.lexeme_arena.resolve(&token.lexeme)
            );
            Node::Leaf(*token)
          }
          None => return Err(Error::ExpectedToken(next_token.span)),
        },
        Action::Reduce(production_id) | Action::Accept(production_id) => {
          let production = self.grammar.production(*production_id);

          if let Action::Reduce(_) = action {
            debug!("Reduce to {}", production.rule);
          }

          // When we decide to reduce to a production P, this
          // entails popping off the N nodes from the stack that
          // its definition entails. Our current state (before
          // pushing P back into the stack) is then exactly the
          // state we were in BEFORE we pushed in the first of
          // these N nodes.
          //
          // Therefore, in each stack entry, we stash the state id
          // that we were in before pushing in that element. So
          // we just inspect the first of these drained elements
          // (stack[drain_from]) to figure out what our next state is.
          if production.is_empty() {
            Node::Parent(Parent {
              rule: production.rule,
              children: vec![],
            })
          } else {
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
        }
      };

      if let Action::Accept(_) = action {
        if !stack.is_empty() {
          return Err(Error::ExcessProgram(next_token.span));
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
          return Err(Error::UnexpectedToken(next_token.span));
        }
      }

      stack.push((prev_state, next_node));
    }
  }
}
