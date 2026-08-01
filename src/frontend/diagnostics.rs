use std::fmt::Display;

use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};

pub use crate::lexer::Span;

pub struct Diagnostic {
  pub message: String,
  pub labels: Vec<Label>,
  pub notes: Vec<String>,
}

impl Diagnostic {
  pub fn from_message(message: impl Display) -> Self {
    Diagnostic {
      message: format!("{message}"),
      labels: Vec::new(),
      notes: Vec::new(),
    }
  }
  pub fn from_span(message: impl Into<String>, span: Span) -> Self {
    Diagnostic {
      message: message.into(),
      labels: vec![Label {
        span,
        message: None,
      }],
      notes: Vec::new(),
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
