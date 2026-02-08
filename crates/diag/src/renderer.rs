use crate::{Diagnostic, LineColumn, SourceManager};

pub trait DiagnosticRenderer {
    fn render(&self, source_manager: &SourceManager, diagnostic: Diagnostic) -> String;
}

pub struct PlainDiagnosticRenderer;

impl DiagnosticRenderer for PlainDiagnosticRenderer {
    fn render(&self, source_manager: &SourceManager, diagnostic: Diagnostic) -> String {
        let mut out = String::new();
        out.push_str(diagnostic.severity.as_str());
        if let Some(code) = &diagnostic.code {
            out.push_str(&format!("[{}]", code));
        }
        out.push_str(&format!(" : {}\n", diagnostic.message));
        for label in &diagnostic.labels {
            let file = source_manager
                .get_file(label.file_id)
                .expect("file not found in SourceManager");
            let span = label.span;
            let start = span.start.0;
            let end = span.end.0;
            let LineColumn { line, col } = file.line_col(span.start);
            let line_start = file.line_starts[line - 1].0;
            let line_end = file
                .line_starts
                .get(line)
                .map(|p| p.0)
                .unwrap_or(file.source.len());
            let line_src = &file.source[line_start..line_end];
            out.push_str(&format!(" --> {}:{}:{}\n", file.name, line, col));
            out.push_str(&format!("{:4} | {}\n", line, line_src.trim_end()));
            let caret_len = (end - start).max(1);
            let caret_pad = col - 1;
            out.push_str("     | ");
            out.push_str(&" ".repeat(caret_pad));
            out.push_str(&"^".repeat(caret_len));
            if let Some(msg) = &label.message {
                out.push_str(&format!(" {}", msg));
            }
            out.push('\n');
        }
        for note in diagnostic.notes {
            out.push_str(&format!("note: {}\n", note));
        }
        if let Some(help) = diagnostic.help {
            out.push_str(&format!("help: {}\n", help));
        }
        out
    }
}
