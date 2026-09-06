use std::fmt::Display;

use crate::parse::Error as ParseError;
use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use lexer::Error as LexError;

pub use lexer::Span;

pub struct Diagnostic {
  pub message: String,
  pub labels: Vec<Label>,
}

impl Diagnostic {
  pub fn from_message(message: impl Display) -> Self {
    Diagnostic {
      message: format!("{message}"),
      labels: Vec::new(),
    }
  }
  pub fn from_span(message: impl Into<String>, span: Span) -> Self {
    Diagnostic {
      message: message.into(),
      labels: vec![Label {
        span,
        message: None,
      }],
    }
  }
}

pub struct Label {
  pub span: Span,
  pub message: Option<String>,
}

pub trait ToDiagnostic {
  fn to_diagnostic(&self) -> Diagnostic;
}

pub struct DiagnosticRenderer<'s> {
  source: &'s str,
  renderer: Renderer,
}

impl<'s> DiagnosticRenderer<'s> {
  pub fn new(source: &'s str) -> Self {
    Self {
      source,
      renderer: Renderer::styled(),
    }
  }

  pub fn render(&self, diag: &Diagnostic) -> String {
    let mut snippet = Snippet::source(self.source);

    for label in &diag.labels {
      snippet = snippet.annotation(
        AnnotationKind::Primary
          .span(label.span.range())
          .label(label.message.as_deref().unwrap_or("")),
      );
    }
    let report = &[Level::ERROR.primary_title(&diag.message).element(snippet)];
    self.renderer.render(report).to_string()
  }
}

impl ToDiagnostic for LexError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::UnterminatedEscape(_) => Diagnostic::from_message(self),
      Self::UnterminatedRegex(_) => Diagnostic::from_message(self),
      Self::MalformattedRange(_) => Diagnostic::from_message(self),
      Self::UnorderedRange(_, _) => Diagnostic::from_message(self),

      &Self::NoMatchingToken(span) => Diagnostic::from_span("No matching token", span),
    }
  }
}

impl ToDiagnostic for ParseError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      &Self::ExpectedToken(span) => Diagnostic::from_span("Expected token", span),
      &Self::UnexpectedToken(span) => Diagnostic::from_span("Unexpected token", span),
      &Self::ExcessProgram(span) => Diagnostic::from_span("Unexpected excess characters", span),
      Self::IncompleteProgram => Diagnostic::from_message("Incomplete program"),
    }
  }
}
