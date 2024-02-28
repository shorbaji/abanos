//! Parser for the abanos programming language
//!
//! A parser for the abanos programming language.
//! Provides Parser struct whith is an Iterator over Result<Expr, ReadError>
//! Implemented mostly based on r7rs small for scheme
mod error;
mod lexer;
#[cfg(test)]
mod tests;

use atty::Stream;
use error::ParseError;
use lexer::Token;
pub use lib::expr::Expr;
use std::io::Write;
use std::iter::Peekable;

/// abanos parser
///
pub struct Parser<R>
where
    R: std::io::BufRead,
{
    lexer: Peekable<lexer::BufferedLexer<R>>,
}

impl<R> Iterator for Parser<R>
where
    R: std::io::BufRead,
{
    type Item = Result<Expr, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        // interactive prompt if stdin is a tty
        if atty::is(Stream::Stdin) {
            print!(">> ");
            std::io::stdout().flush().unwrap();
        }

        // read next token from lexer
        // no more tokens results in no more expressions (None)
        // lexical error results in error and we recover
        // otherwise we parse the expression
        // if we get a parse error we recover
        match self.lexer.peek() {
            None => None,
            Some(Err(e)) => Some(Err((e, 0).into()).inspect_err(|e| self.recover(e))),
            Some(Ok(_)) => Some(self.expr(0).inspect_err(|e| self.recover(e))),
        }
    }
}

/// The parser is an iterator over Result<Expr, ParseError>
/// generic over BufRead reader, e.g. std::io::Stdin::lock()
impl<R> Parser<R>
where
    R: std::io::BufRead,
{
    pub fn new(reader: R) -> Parser<R> {
        Parser {
            lexer: lexer::BufferedLexer::new(reader).peekable(),
        }
    }

    fn recover(&mut self, e: &ParseError) {
        // recover from a parse error
        // the recover strategy is to skip tokens until we find a matching right parenthesis
        // or until we reach the end of the file
        // we implement this by maintaining a counter of left and right parenthesis
        match e {
            ParseError::UnexpectedToken(_, mut r) | ParseError::LexicalError(mut r) => loop {
                let token = self.lexer.next();
                if r == 0 {
                    break;
                }
                match token {
                    Some(Ok(Token::ParenLeft)) => r += 1,
                    Some(Ok(Token::ParenRight)) => {
                        r -= 1;
                        if r == 0 {
                            break;
                        }
                    }
                    None => break,
                    _ => (),
                }
            },
            ParseError::UnexpectedEof => (),
            ParseError::ReadLineError => (),
        }
    }

    /// this is the top level of the parser
    /// it is the entry point for parsing an expression
    /// implemented as a recursive descent parser
    /// we maintain a recovery depth r to allow recovery
    fn expr(&mut self, r: u16) -> Result<Expr, ParseError> {
        match self.peek(r)? {
            Token::Boolean(b) => self.boolean(b),
            Token::Char(c) => self.char(c),
            Token::Number(n) => self.number(n),
            Token::String(s) => self.string(s),
            Token::Quote => self.quotation(r),
            Token::HashU8Open => self.bytevector(r),
            Token::HashOpen => self.vector(r),
            Token::ParenLeft => self.compound(r),
            _ => self.variable(r),
        }
    }

    #[inline]
    fn boolean(&mut self, b: bool) -> Result<Expr, ParseError> {
        self.lexer.next();
        Ok(Expr::Boolean(b))
    }

    #[inline]
    fn char(&mut self, c: char) -> Result<Expr, ParseError> {
        self.lexer.next();
        Ok(Expr::Char(c))
    }

    #[inline]
    fn string(&mut self, s: String) -> Result<Expr, ParseError> {
        self.lexer.next();
        Ok(Expr::String(s))
    }

    #[inline]
    fn number(&mut self, n: String) -> Result<Expr, ParseError> {
        self.lexer.next();
        Ok(Expr::Number(n))
    }

    fn quotation(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.lexer.next();
        self.datum(r)
    }

    fn datum(&mut self, r: u16) -> Result<Expr, ParseError> {
        match self.peek(r)? {
            Token::Boolean(b) => self.boolean(b),
            Token::Char(c) => self.char(c),
            Token::Number(n) => self.number(n),
            Token::String(s) => self.string(s),
            Token::Quote => self.quotation(r),
            Token::HashU8Open => self.bytevector(r),
            Token::HashOpen => self.vector(r),
            Token::ParenLeft => self.compound_datum(r),
            Token::Identifier(_) => self.variable(r),
            t => Err(ParseError::UnexpectedToken(t, r)),
        }
    }

    // a compound datum starts with a left parenthesis
    fn compound_datum(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.lexer.next();
        let v = self.zero_or_more(Parser::datum, r + 1)?;
        self.paren_right(r + 1)?;
        Ok(Expr::List(v))
    }

    #[inline]
    fn bytevector(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.lexer.next();
        let v = self.zero_or_more(Parser::byte, r + 1)?;
        self.paren_right(r + 1)?;

        Ok(Expr::Bytevector(v))
    }

    #[inline]
    fn byte(&mut self, r: u16) -> Result<u8, ParseError> {
        match self.peek(r)? {
            Token::Number(n) => match n.parse::<u8>() {
                Ok(b) => {
                    self.lexer.next();
                    Ok(b)
                }
                Err(_) => Err(ParseError::UnexpectedToken(Token::Number(n), r)),
            },
            t => Err(ParseError::UnexpectedToken(t, r)),
        }
    }

    #[inline]
    fn vector(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.lexer.next();
        let v = self.zero_or_more(Parser::datum, r + 1)?;
        self.paren_right(r + 1)?;
        Ok(Expr::Vector(v))
    }

    #[inline]
    fn variable(&mut self, r: u16) -> Result<Expr, ParseError> {
        match self.peek(r)? {
            Token::Identifier(id) => {
                self.lexer.next();
                Ok(Expr::Variable(id))
            }
            t => Err(ParseError::UnexpectedToken(t, r)),
        }
    }

    // a compound expression starts with a left parenthesis
    // it is either one of the special forms or otherwise an application
    fn compound(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.paren_left(r)?;

        match self.peek(r + 1)? {
            Token::Identifier(id) if id == "define" => self.definition(r + 1),
            Token::Identifier(id) if id == "if" => self.conditional(r + 1),
            Token::Identifier(id) if id == "lambda" => self.lambda(r + 1),
            Token::Identifier(id) if id == "quote" => self.long_quotation(r + 1),
            Token::Identifier(id) if id == "set!" => self.assignment(r + 1),
            _ => self.application(r + 1),
        }
    }

    fn definition(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.lexer.next(); // consume define

        match self.peek(r)? {
            Token::ParenLeft => self.define_lambda(r),
            _ => self.define_variable(r),
        }
    }

    fn define_lambda(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.paren_left(r)?;
        let symbol = self.variable(r + 1)?;
        Ok(Expr::Define(
            Box::new(symbol),
            Box::new(self.formals_and_body(r + 1)?),
        ))
    }

    fn define_variable(&mut self, r: u16) -> Result<Expr, ParseError> {
        match self.peek(r)? {
            Token::Identifier(id) => {
                self.lexer.next(); // consume identifier
                let symbol = Expr::Variable(id);
                let expr = self.expr(r)?;
                self.paren_right(r)?;
                Ok(Expr::Define(Box::new(symbol), Box::new(expr)))
            }
            t => Err(ParseError::UnexpectedToken(t, r)),
        }
    }

    fn conditional(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.lexer.next(); // consume if

        let predicate = self.expr(r)?;
        let consequent = self.expr(r)?;
        let alternative = self.expr(r)?;

        self.paren_right(r)?;
        Ok(Expr::If(
            Box::new(predicate),
            Box::new(consequent),
            Box::new(alternative),
        ))
    }

    fn lambda(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.lexer.next(); // consume lambda
        self.paren_left(r)?;
        self.formals_and_body(r + 1)
    }

    // used by (define (foo ...)) and by lambda
    fn formals_and_body(&mut self, r: u16) -> Result<Expr, ParseError> {
        let formals = self.zero_or_more(Parser::expr, r)?;
        self.paren_right(r)?;
        let body = self.zero_or_more(Parser::expr, r - 1)?;
        self.paren_right(r - 1)?;
        Ok(Expr::Lambda(formals, body))
    }

    fn long_quotation(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.lexer.next(); // consume quote
        let d = self.datum(r)?;
        self.paren_right(r)?;

        Ok(d)
    }

    fn assignment(&mut self, r: u16) -> Result<Expr, ParseError> {
        self.lexer.next(); // consume set!
        let v = self.expr(r)?;
        let e = self.expr(r)?;
        self.paren_right(r)?;
        Ok(Expr::Set(Box::new(v), Box::new(e)))
    }

    // (<operator> <operand>*)
    fn application(&mut self, r: u16) -> Result<Expr, ParseError> {
        let operator = self.expr(r)?;
        let operands = self.zero_or_more(Parser::expr, r)?;
        self.paren_right(r)?;
        Ok(Expr::Apply(Box::new(operator), operands))
    }

    #[inline]
    fn paren_left(&mut self, r: u16) -> Result<(), ParseError> {
        match self.peek(r)? {
            Token::ParenLeft => {
                self.lexer.next();
                Ok(())
            }
            t => Err(ParseError::UnexpectedToken(t, r)),
        }
    }

    #[inline]
    fn paren_right(&mut self, r: u16) -> Result<(), ParseError> {
        match self.peek(r)? {
            Token::ParenRight => {
                self.lexer.next();
                Ok(())
            }
            t => Err(ParseError::UnexpectedToken(t, r)),
        }
    }

    // utility functions
    // peek next token from lexer and raise error if EOF
    // need r: recovery depth in case we get a lexical error
    fn peek(&mut self, r: u16) -> Result<lexer::Token, ParseError> {
        match self.lexer.peek().cloned() {
            Some(Ok(token)) => Ok(token),
            Some(Err(e)) => Err((&e, r).into()),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    // zero or more <T> items by calling a function f while it returns Ok(<T>)
    fn zero_or_more<T>(
        &mut self,
        f: fn(&mut Self, u16) -> Result<T, ParseError>,
        r: u16,
    ) -> Result<Vec<T>, ParseError> {
        Ok(std::iter::repeat_with(|| f(self, r))
            .take_while(|result| result.is_ok())
            .map(|result| result.unwrap())
            .collect())
    }
}
