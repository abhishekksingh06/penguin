use std::ops::Range;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct BytePos(pub usize);

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Span {
    pub start: BytePos,
    pub end: BytePos,
}

impl Span {
    pub fn new(start: BytePos, end: BytePos) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }

    pub fn from_range(range: Range<usize>) -> Self {
        Self::new(BytePos(range.start), BytePos(range.end))
    }

    pub fn len(&self) -> usize {
        self.end.0 - self.start.0
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            start: BytePos(self.start.0.min(other.start.0)),
            end: BytePos(self.end.0.max(other.end.0)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    pub fn map<F, U>(self, f: F) -> Spanned<U>
    where
        F: Fn(T) -> U,
    {
        Spanned {
            span: self.span,
            inner: f(self.inner),
        }
    }
}
