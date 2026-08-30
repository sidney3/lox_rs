use super::first::first;
use super::{Grammar, Rule, Symbol};
use crate::lexer::TokenType;
use itertools::Itertools;
use log::debug;
use smallvec::SmallVec;
use std::collections::HashSet;

pub(super) fn follow<R: Rule>(grammar: &Grammar<R>) -> Vec<HashSet<R::TokenType>> {
  let mut changed = true;
  let mut follow_table: Vec<HashSet<R::TokenType>> =
    (0..R::COUNT).map(|_| HashSet::new()).collect();

  let _initial_symbols: Vec<_> = grammar
    .productions()
    .filter_map(|production| match production.definition.first() {
      Some(&Symbol::Token(t)) => Some(t),
      Some(Symbol::Rule(_)) => None,
      _ => None,
    })
    .collect();

  follow_table[grammar.target_rule().ord()].insert(R::TokenType::eof());
  let first_table = first(grammar);

  // classic fixed point calculation
  while changed {
    changed = false;
    for production in grammar.productions() {
      let follows: SmallVec<[R::TokenType; 10]> = follow_table[production.rule.ord()]
        .iter()
        .cloned()
        .collect();

      let mut add_follow = |rule: &R, token: R::TokenType| {
        if !follow_table[rule.ord()].contains(&token) {
          changed = true;
          follow_table[rule.ord()].insert(token);
        }
      };

      let def = &production.definition;

      // Handling here with epsilon productions is tricky.
      //
      // We would like to find all rules A and tokens 'T' such that
      // there exists a partial expansion of this production containing
      // A'T'.
      //
      // A naive approach would be to walk through all pairs of symbols (a,b)
      // and add FIRST(b) to FOLLOW(a). This fails when b can be EPSILON - we
      // should then add FIRST(c) (where c follows b). And if c is EPSILON...
      let rule_contains_epsilon = |r| {
        grammar
          .productions_for_rule(r)
          .iter()
          .any(|&p| grammar.production(p).is_epsilon())
      };
      for (i, sym) in def.iter().enumerate() {
        let Symbol::Rule(r) = sym else { continue };
        for following in def[i + 1..].iter().cloned() {
          match following {
            Symbol::Token(t) => {
              add_follow(r, t);
              break;
            }
            Symbol::Rule(r2) => {
              for f in &first_table[r2.ord()] {
                add_follow(r, *f);
              }

              if !rule_contains_epsilon(r2) {
                break;
              }
            }
          }
        }
      }

      // Suppose we have
      //
      // alpha = 'u';
      // beta = '(' alpha ')'
      //
      // Our stack is '(', 'u', and the next token is ')'. Then clearly
      // we should reduce to alpha, so ')' should be in follow(alpha) -
      //
      // rigorously, we say that if
      //
      // A = ....B;
      //
      // Then B should contain FOLLOW(A)
      if let Some(Symbol::Rule(r)) = production.definition.last() {
        for t in follows {
          add_follow(r, t);
        }
      }
    }
  }

  for r in grammar.rules() {
    let follow = &follow_table[r.ord()];

    debug!("Follow({r:?}) := {follow:?}");
  }

  follow_table
}
