use crate::{BytePos, Span};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub struct FileId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub struct LineColumn {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub struct SourceFile {
    pub name: String,
    pub source: String,
    pub line_starts: Vec<BytePos>,
}

impl SourceFile {
    pub fn new(name: String, source: String, line_starts: Vec<BytePos>) -> Self {
        Self {
            name,
            source,
            line_starts,
        }
    }

    pub fn source(&self, span: Span) -> Option<&str> {
        self.source.get(span.start.0..span.end.0)
    }

    pub fn line_col(&self, pos: BytePos) -> LineColumn {
        let line = match self.line_starts.binary_search(&pos) {
            Ok(i) => i,
            Err(0) => 0,
            Err(i) => i - 1,
        };
        let col = pos.0 - self.line_starts[line].0;
        LineColumn {
            line: line + 1,
            col: col + 1,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq)]
pub struct SourceManager {
    files: Vec<SourceFile>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, name: String, source: String) -> FileId {
        let line_starts = compute_line_starts(&source);
        let file_id = FileId(self.files.len());
        let source = SourceFile::new(name, source, line_starts);
        self.files.push(source);
        file_id
    }

    pub fn get_file(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.0)
    }
}

fn compute_line_starts(source: &str) -> Vec<BytePos> {
    let mut bytes = vec![BytePos(0)];
    for (idx, ch) in source.char_indices() {
        if ch == '\n' {
            bytes.push(BytePos(idx + 1));
        }
    }
    bytes
}
