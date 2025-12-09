use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("Compilation error")]
pub enum Error {
    #[error("Lexer error: {message}")]
    #[diagnostic(code(rux::lexer))]
    Lexer {
        message: String,
        source_code: String,
        #[label("here")]
        span: SourceSpan,
    },
    
    #[error("Parser error: {message}")]
    #[diagnostic(code(rux::parser))]
    Parser {
        message: String,
        source_code: String,
        #[label("here")]
        span: SourceSpan,
    },
    
    #[error("Type error: {message}")]
    #[diagnostic(code(rux::type_check))]
    Type {
        message: String,
        source_code: String,
        #[label("here")]
        span: SourceSpan,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn lexer(message: impl Into<String>, source: impl Into<String>, span: SourceSpan) -> Self {
        Error::Lexer {
            message: message.into(),
            source_code: source.into(),
            span,
        }
    }
    
    pub fn parser(message: impl Into<String>, source: impl Into<String>, span: SourceSpan) -> Self {
        Error::Parser {
            message: message.into(),
            source_code: source.into(),
            span,
        }
    }
    
    pub fn type_error(message: impl Into<String>, source: impl Into<String>, span: SourceSpan) -> Self {
        Error::Type {
            message: message.into(),
            source_code: source.into(),
            span,
        }
    }
}
