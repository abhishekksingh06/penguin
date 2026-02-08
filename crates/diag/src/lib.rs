mod renderer;
mod source;
mod span;

pub use renderer::*;
pub use source::*;
pub use span::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

impl Severity {
    fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Label {
    pub span: Span,
    pub file_id: FileId,
    pub message: Option<String>,
    pub is_primary: bool,
}

impl Label {
    pub fn primary(file_id: FileId, span: Span) -> Self {
        Self {
            span,
            file_id,
            message: None,
            is_primary: true,
        }
    }

    pub fn secondary(file_id: FileId, span: Span) -> Self {
        Self {
            span,
            file_id,
            message: None,
            is_primary: false,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Severity,
    pub code: Option<String>,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn new(severity: Severity) -> Self {
        Self {
            severity,
            message: String::new(),
            code: None,
            labels: Vec::new(),
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_notes(mut self, mut notes: Vec<String>) -> Self {
        self.notes.append(&mut notes);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_labels(mut self, mut labels: Vec<Label>) -> Self {
        self.labels.append(&mut labels);
        self
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

pub trait DiagnosticConvertible {
    fn into_diagnostic(self) -> Diagnostic;
}
