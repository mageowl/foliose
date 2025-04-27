use crate::span::Span;

#[derive(Debug)]
pub struct Error {
    message: String,
    span: Span,
}

impl Error {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    pub fn display(&self, src: &str) {
        println!("error: {}", self.message);
        let lines = src
            .lines()
            .skip(self.span.start.ln)
            .take(self.span.end.ln - self.span.start.ln + 1)
            .enumerate();
        let len = lines.size_hint().1.map(|l| l - 1);
        for (i, line) in lines {
            let first = i == 0;
            let last = Some(i) == len;
            if first && last {
                println!(
                    "{}\x1b[1;31m{}\x1b[0m{}",
                    &line[0..self.span.start.col],
                    &line[self.span.start.col..self.span.end.col],
                    &line[self.span.end.col..]
                );
            } else if first {
                println!(
                    "{}\x1b[1;31m{}",
                    &line[0..self.span.start.col],
                    &line[self.span.start.col..],
                );
            } else if last {
                println!(
                    "{}\x1b[0m{}",
                    &line[0..self.span.end.col],
                    &line[self.span.end.col..],
                );
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! type_error {
    ($expected: expr, $found: expr) => {
        format!("Expected {}, but instead found {}", $expected, $found);
    };
}
