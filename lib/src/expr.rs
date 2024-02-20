// abanos expressions
use serde::{Deserialize, Serialize};

/// Expr
/// represents an expression in the abanos language
/// based on r7rs small
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Expr {
    Apply(Box<Expr>, Vec<Expr>),
    Set(Box<Expr>, Box<Expr>),
    Boolean(bool),
    Bytevector(Vec<u8>),
    Char(char),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Define(Box<Expr>, Box<Expr>),
    Lambda(Vec<Expr>, Vec<Expr>),
    List(Vec<Expr>),
    Number(String),
    String(String),
    Variable(String),
    Vector(Vec<Expr>),
}
