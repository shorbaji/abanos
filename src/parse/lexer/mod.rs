#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::iter::Peekable;

use logos::{Lexer, Logos};

/// Token
/// lexical analyzer based on r7rs small 
#[derive(Serialize, Deserialize, Logos, Debug, Clone, PartialEq)]
pub enum Token {
    #[regex(r"(#(([tT][rR][uU][eE])|([fF][aA][lL][sS][eE])|([tT]|[fF])))", to_bool)]
    Boolean(bool),
    #[regex(r"((#\\x([0-9a-fA-F]+))|(#\\(alarm|backspace|delete|escape|newline|null|return|space|tab))|(#\\.))", to_char)]
    Char(char),
    #[regex(r",")]
    Comma,
    #[regex(r",@")]
    CommaAt,
    #[regex(r"\.")]
    Dot,
    #[regex(r"(;[^(\r\n|\r|\n)]*)", logos::skip)]
    Comment,
    #[regex(r"((#!fold-case)|(#!no-fold-case))")]
    Directive,
    #[regex(
        r"(([a-zA-Z]|[!\$%&\*/:<=>\?\^_~])(([a-zA-Z]|[!\$%&\*/:<=>\?\^_~])|[0-9]|((\+|-)|\.|@))*)",
        to_identifier
    )]
    #[regex(r"(((\+|-)|((\+|-)(([a-zA-Z]|[!\$%&\*/:<=>\?\^_~])|(\+|-)|@)(([a-zA-Z]|[!\$%&\*/:<=>\?\^_~])|[0-9]|((\+|-)|\.|@))*)|((\+|-)\.((([a-zA-Z]|[!\$%&\*/:<=>\?\^_~])|(\+|-)|@)|\.)(([a-zA-Z]|[!\$%&\*/:<=>\?\^_~])|[0-9]|((\+|-)|\.|@))*)|(\.((([a-zA-Z]|[!\$%&\*/:<=>\?\^_~])|(\+|-)|@)|\.)(([a-zA-Z]|[!\$%&\*/:<=>\?\^_~])|[0-9]|((\+|-)|\.|@))*)))",
            to_identifier)]
    Identifier(String),
    #[regex(
        r"(\|([^\|\\]|(\\x([0-9a-fA-F]+);)|(\\[aA]|\\[bB]|\\[tT]|\\[nN]|\\[rR])|(\\\|))*\|)",
        to_identifier
    )]
    VerticalLineIdentifier(String),
    // prioritize Number over Identifier since +i and -i are valid identifiers and numbers according to the r7rs spec
    #[regex(r"(((((#b)((#[eEiI])?))|(((#[eEiI])?)(#b)))((((((\+|-)?)(((((0|1))+)/(((0|1))+))|(((0|1))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+|-)(((((0|1))+)/(((0|1))+))|(((0|1))+))(i|I))|(((((\+|-)?)(((((0|1))+)/(((0|1))+))|(((0|1))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)(i|I))|(((((\+|-)?)(((((0|1))+)/(((0|1))+))|(((0|1))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+|-)(i|I))|(((((\+|-)?)(((((0|1))+)/(((0|1))+))|(((0|1))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))@((((\+|-)?)(((((0|1))+)/(((0|1))+))|(((0|1))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)))|((\+|-)(((((0|1))+)/(((0|1))+))|(((0|1))+))(i|I))|((\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)(i|I))|(((((\+|-)?)(((((0|1))+)/(((0|1))+))|(((0|1))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)))|((\+|-)(i|I))))|((((#o)((#[eEiI])?))|(((#[eEiI])?)(#o)))((((((\+|-)?)(((([0-7])+)/(([0-7])+))|(([0-7])+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+|-)(((([0-7])+)/(([0-7])+))|(([0-7])+))(i|I))|(((((\+|-)?)(((([0-7])+)/(([0-7])+))|(([0-7])+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)(i|I))|(((((\+|-)?)(((([0-7])+)/(([0-7])+))|(([0-7])+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+|-)(i|I))|(((((\+|-)?)(((([0-7])+)/(([0-7])+))|(([0-7])+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))@((((\+|-)?)(((([0-7])+)/(([0-7])+))|(([0-7])+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)))|((\+|-)(((([0-7])+)/(([0-7])+))|(([0-7])+))(i|I))|((\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)(i|I))|(((((\+|-)?)(((([0-7])+)/(([0-7])+))|(([0-7])+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)))|((\+|-)(i|I))))|(((((#d)?)((#[eEiI])?))|(((#[eEiI])?)((#d)?)))((((((\+|-)?)(((((([0-9]))+)/((([0-9]))+))|((([0-9]))+))|((((([0-9]))+)(((e|E)((\+|-)?)(([0-9])+))?))|(\.([0-9])+(((e|E)((\+|-)?)(([0-9])+))?))|(([0-9])+\.([0-9])*(((e|E)((\+|-)?)(([0-9])+))?)))))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+|-)(((((([0-9]))+)/((([0-9]))+))|((([0-9]))+))|((((([0-9]))+)(((e|E)((\+|-)?)(([0-9])+))?))|(\.([0-9])+(((e|E)((\+|-)?)(([0-9])+))?))|(([0-9])+\.([0-9])*(((e|E)((\+|-)?)(([0-9])+))?))))(i|I))|(((((\+|-)?)(((((([0-9]))+)/((([0-9]))+))|((([0-9]))+))|((((([0-9]))+)(((e|E)((\+|-)?)(([0-9])+))?))|(\.([0-9])+(((e|E)((\+|-)?)(([0-9])+))?))|(([0-9])+\.([0-9])*(((e|E)((\+|-)?)(([0-9])+))?)))))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)(i|I))|(((((\+|-)?)(((((([0-9]))+)/((([0-9]))+))|((([0-9]))+))|((((([0-9]))+)(((e|E)((\+|-)?)(([0-9])+))?))|(\.([0-9])+(((e|E)((\+|-)?)(([0-9])+))?))|(([0-9])+\.([0-9])*(((e|E)((\+|-)?)(([0-9])+))?)))))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+|-)(i|I))|(((((\+|-)?)(((((([0-9]))+)/((([0-9]))+))|((([0-9]))+))|((((([0-9]))+)(((e|E)((\+|-)?)(([0-9])+))?))|(\.([0-9])+(((e|E)((\+|-)?)(([0-9])+))?))|(([0-9])+\.([0-9])*(((e|E)((\+|-)?)(([0-9])+))?)))))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))@((((\+|-)?)(((((([0-9]))+)/((([0-9]))+))|((([0-9]))+))|((((([0-9]))+)(((e|E)((\+|-)?)(([0-9])+))?))|(\.([0-9])+(((e|E)((\+|-)?)(([0-9])+))?))|(([0-9])+\.([0-9])*(((e|E)((\+|-)?)(([0-9])+))?)))))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)))|((\+|-)(((((([0-9]))+)/((([0-9]))+))|((([0-9]))+))|((((([0-9]))+)(((e|E)((\+|-)?)(([0-9])+))?))|(\.([0-9])+(((e|E)((\+|-)?)(([0-9])+))?))|(([0-9])+\.([0-9])*(((e|E)((\+|-)?)(([0-9])+))?))))(i|I))|((\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)(i|I))|(((((\+|-)?)(((((([0-9]))+)/((([0-9]))+))|((([0-9]))+))|((((([0-9]))+)(((e|E)((\+|-)?)(([0-9])+))?))|(\.([0-9])+(((e|E)((\+|-)?)(([0-9])+))?))|(([0-9])+\.([0-9])*(((e|E)((\+|-)?)(([0-9])+))?)))))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)))|((\+|-)(i|I))))|((((#x)((#[eEiI])?))|(((#[eEiI])?)(#x)))((((((\+|-)?)((((([0-9a-fA-F]))+)/((([0-9a-fA-F]))+))|((([0-9a-fA-F]))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+|-)((((([0-9a-fA-F]))+)/((([0-9a-fA-F]))+))|((([0-9a-fA-F]))+))(i|I))|(((((\+|-)?)((((([0-9a-fA-F]))+)/((([0-9a-fA-F]))+))|((([0-9a-fA-F]))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)(i|I))|(((((\+|-)?)((((([0-9a-fA-F]))+)/((([0-9a-fA-F]))+))|((([0-9a-fA-F]))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))(\+|-)(i|I))|(((((\+|-)?)((((([0-9a-fA-F]))+)/((([0-9a-fA-F]))+))|((([0-9a-fA-F]))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0))@((((\+|-)?)((((([0-9a-fA-F]))+)/((([0-9a-fA-F]))+))|((([0-9a-fA-F]))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)))|((\+|-)((((([0-9a-fA-F]))+)/((([0-9a-fA-F]))+))|((([0-9a-fA-F]))+))(i|I))|((\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)(i|I))|(((((\+|-)?)((((([0-9a-fA-F]))+)/((([0-9a-fA-F]))+))|((([0-9a-fA-F]))+)))|(\+inf\.0|-inf\.0|\+nan\.0|-nan\.0|\+INF\.0|-INF\.0|\+NAN\.0|-NAN\.0)))|((\+|-)(i|I)))))",
        to_number,
        priority=3)]
    Number(String),
    #[regex(r"\(")]
    ParenLeft,
    #[regex(r"\)")]
    ParenRight,
    #[regex(r"`")]
    Quasiquote,
    #[regex(r"'")]
    Quote,
    #[regex(r"#\(")]
    HashOpen,
    #[regex(r"#u8\(")]
    HashU8Open,
    #[regex(r#""([^"\\]|(\\[aA]|\\[bB]|\\[tT]|\\[nN]|\\[rR])|\\"|\\|\\( |\t)*(\r\n|\r|\n)( |\t)*|(\\x([0-9a-fA-F]+);))*""#, to_string)]
    String(String),
    #[regex(r"(( |\t)|(\r\n|\r|\n))")]
    Whitespace,
}

fn to_bool(lex: &mut Lexer<Token>) -> Option<bool> {
    let s = lex.slice();
    match s {
        "#t" | "#T" | "#true" | "#True" | "#TRUE" => Some(true),
        "#f" | "#F" | "#false" | "#False" | "#FALSE" => Some(false),
        _ => None,
    }
}

fn to_char(lex: &mut Lexer<Token>) -> Option<char> {
    let s = lex.slice();

    if s.len() == 3 {
        return Some(s[2..].chars().next().unwrap());
    } else {
        match &s[0..3] {
            "#\\x" => {
                let hex = &s[3..];
                let hex = u32::from_str_radix(hex, 16).unwrap();
                let c = std::char::from_u32(hex).unwrap();
                Some(c)
            }
            _ => match &s[2..] {
                "alarm" => Some('\u{0007}'),
                "backspace" => Some('\u{0008}'),
                "delete" => Some('\u{007F}'),
                "escape" => Some('\u{001B}'),
                "newline" => Some('\u{000A}'),
                "null" => Some('\u{0000}'),
                "return" => Some('\u{000D}'),
                "space" => Some('\u{0020}'),
                "tab" => Some('\u{0009}'),
                _ => None,
            },
        }
    }
}

fn to_identifier(lex: &mut Lexer<Token>) -> Option<String> {
    Some(lex.slice().to_string())
}

fn to_number(lex: &mut Lexer<Token>) -> Option<String> {
    Some(lex.slice().to_string())
}

fn to_string(lex: &mut Lexer<Token>) -> Option<String> {
    let s = lex.slice();

    let s: String = String::from(&s[1..s.len() - 1]);
    Some(s)
}

/// DLexer
/// Implements delimiting
/// From r7rs small: "Identifiers that do not begin with a vertical line are 
/// terminated by a delimiter or by the end of the input."
/// dot, numbers, characters, and booleans"
pub struct DLexer<'a> {
    lexer: Peekable<Lexer<'a, Token>>,
}

impl<'a> DLexer<'a> {
    /// This function returns a new lexer for the given input.
    /// It creates a Logos lexer and wraps it in a Peekable iterator.
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Token::lexer(input).peekable(),
        }
    }
}

impl Iterator for DLexer<'_> {
    type Item = Result<Token, ()>;

    /// This function returns the next token in the lexer.
    ///
    /// Boolean, character, directive, dot, identifier (without vertical lines), number must
    /// be terminated by a delimiter.
    ///
    fn next(&mut self) -> Option<Self::Item> {
        // If the next token is a boolean, character, directive, dot, identifier (without vertical lines), number
        // then we need to check if the lexeme after it starts with a delimiter.
        match self.lexer.peek()? {
            Err(_) => Some(Err(())),
            Ok(t) => {
                match t {
                    Token::Boolean(_)
                    | Token::Char(_)
                    | Token::Dot
                    | Token::Directive
                    | Token::Identifier(_)
                    | Token::Number(_) => {
                        let token = self.lexer.next()?; // Consume the token

                        match self.lexer.peek() {
                            Some(next) => match next {
                                Err(_) => Some(token),
                                Ok(t) => match t {
                                    Token::Whitespace
                                    | Token::ParenRight
                                    | Token::ParenLeft
                                    | Token::String(_)
                                    | Token::Comment
                                    | Token::VerticalLineIdentifier(_) => Some(token),
                                    _ => {
                                        self.lexer.next();
                                        Some(Err(()))
                                    }
                                },
                            },
                            None => Some(token),
                        }
                    }
                    Token::VerticalLineIdentifier(_) => self.lexer.next(),
                    Token::Whitespace => {
                        while let Some(Ok(Token::Whitespace)) = self.lexer.peek() {
                            self.lexer.next();
                        }
                        self.lexer.next()
                    }
                    _ => self.lexer.next(),
                }
            }
        }
    }
}

/// BufferedLexer
/// This struct wraps a DLexer and provides a buffered interface to it.
/// It implements a refresh to read from a BufRead and fill the token buffer.
/// until EOF is reached.
pub struct BufferedLexer<R>
where
    R: std::io::BufRead,
{
    reader: R,
    tokens: VecDeque<Result<Token, ()>>,
}

impl<R> BufferedLexer<R>
where
    R: std::io::BufRead,
{
    pub fn new(reader: R) -> BufferedLexer<R> {
        BufferedLexer {
            tokens: VecDeque::new(),
            reader,
        }
    }

    /// Refresh the buffer queue of tokens if empty
    pub fn refresh(&mut self) -> Result<(), std::io::Error> {
        loop {
            if self.tokens.is_empty() {
                let mut buffer = String::new();
                match self.reader.read_line(&mut buffer) {
                    Ok(0) => break Ok(()), // EOF so we break the loop and leave the token buffer empty
                    Ok(_) => {
                        // a line (possibly empty so we do not break the loop)
                        let lxr = DLexer::new(buffer.as_str()).peekable();
                        self.tokens = lxr.collect();
                    }
                    Err(e) => break Err(e),
                }
            } else {
                break Ok(());
            }
        }
    }
}

impl<R> Iterator for BufferedLexer<R>
where
    R: std::io::BufRead,
{
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.refresh() {
            Ok(_) => self
                .tokens
                .pop_front()
                .map(|rt| rt.map_err(|_| LexerError::LexicalError)),
            Err(_) => Some(Err(LexerError::ReadLineError)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LexerError {
    LexicalError,
    ReadLineError,
}
