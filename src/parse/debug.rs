use std::fmt;

use super::Grammar;
use super::Rule;

pub trait DisplayWithGrammar<R: Rule>: Sized {
  fn fmt_with(&self, grammar: &Grammar<R>, f: &mut fmt::Formatter) -> fmt::Result;
}

pub trait DisplayWithGrammarExt<R: Rule>: DisplayWithGrammar<R> {
  fn with<'a, 'b>(&'a self, grammar: &'b Grammar<R>) -> GrammarWrapper<'a, 'b, R, Self>
  where
    Self: Sized,
  {
    GrammarWrapper {
      grammar,
      value: self,
    }
  }
}

impl<R: Rule, T: DisplayWithGrammar<R> + ?Sized> DisplayWithGrammarExt<R> for T {}

pub struct GrammarWrapper<'a, 'b, R: Rule, T> {
  value: &'a T,
  grammar: &'b Grammar<R>,
}

impl<R: Rule, T: DisplayWithGrammar<R>> fmt::Display for GrammarWrapper<'_, '_, R, T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.value.fmt_with(self.grammar, f)
  }
}
