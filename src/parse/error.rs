use crate::parse::lexer;
use serde::{Deserialize, Serialize};

/// ParseError
///
/// This enum represents the different types of errors that can occur during parsing.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(String, u16),
    UnexpectedEof,
    LexicalError(u16),
    ReadLineError,
}

impl From<(&lexer::LexerError, u16)> for ParseError {
    fn from((e, r): (&lexer::LexerError, u16)) -> Self {
        match e {
            lexer::LexerError::LexicalError => ParseError::LexicalError(r),
            lexer::LexerError::ReadLineError => ParseError::ReadLineError,
        }
    }
}
