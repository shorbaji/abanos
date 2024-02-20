use crate::builtin::Builtin;
use crate::continuation::Closure;
use crate::env::Env;
use crate::expr::Expr;
use crate::user::User;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Value
/// represents a value or object in the language
/// continuations and environments are first-class objects
/// and so is a User
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Value {
    Boolean(bool),
    Bytevector(Vec<u8>),
    Char(char),
    Lambda(Vec<Expr>, Vec<Expr>, #[serde(skip)] Arc<Mutex<Env>>),
    List(Vec<Value>),
    Null,
    Number(String),
    String(String),
    Symbol(String),
    Vector(Vec<Value>),

    Builtin(Builtin),
    Continuation(Closure),
    Env(Env),
    User(User),
}

impl TryFrom<Expr> for Value {
    type Error = String;

    fn try_from(e: Expr) -> Result<Self, Self::Error> {
        match e {
            Expr::Boolean(b) => Ok(Value::Boolean(b)),
            Expr::Bytevector(b) => Ok(Value::Bytevector(b)),
            Expr::Char(c) => Ok(Value::Char(c)),
            Expr::List(l) => Ok(Value::List(
                l.into_iter()
                    .map(Value::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Expr::Number(n) => Ok(Value::Number(n)),
            Expr::String(s) => Ok(Value::String(s)),
            Expr::Vector(v) => Ok(Value::Vector(
                v.into_iter()
                    .map(Value::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Expr::Variable(v) => Ok(Value::Symbol(v)),
            _ => Err("invalid list element".to_string()),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Bytevector(b) => write!(
                f,
                "#u8({})",
                b.iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Value::Char(c) => write!(f, "#\\{}", c),
            Value::Lambda(_, _, _) => write!(f, "#<procedure>"),
            Value::List(l) => write!(
                f,
                "({})",
                l.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Value::Null => write!(f, "()"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Vector(v) => write!(
                f,
                "#({})",
                v.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Value::User(u) => write!(f, "#<user:{}>", u.name),
            Value::Env(_) => write!(f, "#<env>"),
            Value::Continuation(_) => write!(f, "#<continuation>"),
            Value::Builtin(b) => write!(f, "#<builtin:{}>", b.name),
        }
    }
}
