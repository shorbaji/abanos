use crate::parse::lexer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ReadError {
    UnexpectedToken(String, u16),
    UnexpectedEof,
    LexicalError(u16),
    ReadLineError,
}

impl From<(&lexer::LexerError, u16)> for ReadError {
    fn from((e, r): (&lexer::LexerError, u16)) -> Self {
        match e {
            lexer::LexerError::LexicalError => ReadError::LexicalError(r),
            lexer::LexerError::ReadLineError => ReadError::ReadLineError,
        }
    }
}
