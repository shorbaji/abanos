//! Abanos continuations
//! provides Arg, Context, Closure, and Continuation
use crate::{env::Env, expr::Expr, user::User, value::Value};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Arg - represents possible argument to an eval function
/// - None - no argument
/// - Expr - an expression
/// - Value - a value
/// - ExprList - a list of expressions
/// - ValueList - a list of values
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Arg {
    None,
    Expr(Expr),
    Value(Value),
    ExprList(Vec<Expr>),
    ValueList(Vec<Value>),
}

impl TryInto<Expr> for Arg {
    type Error = String;

    fn try_into(self) -> Result<Expr, Self::Error> {
        match self {
            Arg::Expr(expr) => Ok(expr),
            _ => Err("expected expression".to_string()),
        }
    }
}

impl TryInto<Value> for Arg {
    type Error = String;

    fn try_into(self) -> Result<Value, Self::Error> {
        match self {
            Arg::Value(value) => Ok(value),
            _ => Err("expected value".to_string()),
        }
    }
}

impl TryInto<Vec<Expr>> for Arg {
    type Error = String;

    fn try_into(self) -> Result<Vec<Expr>, Self::Error> {
        match self {
            Arg::ExprList(exprs) => Ok(exprs),
            _ => Err("expected list of expressions".to_string()),
        }
    }
}

impl TryInto<Vec<Value>> for Arg {
    type Error = String;

    fn try_into(self) -> Result<Vec<Value>, Self::Error> {
        match self {
            Arg::ValueList(values) => Ok(values),
            _ => Err("expected list of values".to_string()),
        }
    }
}

/// Context - represents the context in which an expression is evaluated
/// - r - the lexical environment
/// - d - the dynamic environment
/// - user - the user 
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    #[serde(skip)]
    pub r: Arc<Mutex<Env>>,
    #[serde(skip)]
    pub d: Arc<Mutex<Env>>,
    pub user: User,
}

impl Context {
    pub async fn rget(
        &self,
        symbol: &String,
        r: Arc<Mutex<Env>>,
        k: Box<Closure>,
    ) -> Result<Continuation, String> {
        r.lock().await.get(symbol, self, k).await
    }
}

/// Closure
/// represents a closure of a continuation
/// each variant corresponds to a function
/// and holds:
/// 1. captured variables,
/// 1. the context in which the content will run
/// 1. the closure for the next continuation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Closure {
    Apply {
        operator: Box<Expr>,
        operands: Vec<Expr>,
        context: Context,
        k: Option<Box<Closure>>,
    },
    Call {
        operator: Box<Value>,
        context: Context,
        k: Option<Box<Closure>>,
    },
    Define {
        symbol: Box<Expr>,
        e: Box<Expr>,
        context: Context,
        k: Option<Box<Closure>>,
    },
    DefineAfter {
        symbol: Expr,
        context: Context,
        k: Option<Box<Closure>>,
    },
    Eval {
        context: Context,
        k: Option<Box<Closure>>,
    },
    EvalBody {
        context: Context,
        k: Option<Box<Closure>>,
    },
    EvalBodyAfter {
        body: Vec<Expr>,
        context: Context,
        k: Option<Box<Closure>>,
    },
    EvalOperatorAfter {
        operands: Vec<Expr>,
        context: Context,
        k: Option<Box<Closure>>,
    },
    Evlis {
        context: Context,
        k: Option<Box<Closure>>,
    },
    EvlisAfter {
        exprs: Vec<Expr>,
        acc: Vec<Value>,
        context: Context,
        k: Option<Box<Closure>>,
    },
    If {
        predicate: Box<Expr>,
        consequent: Box<Expr>,
        alternative: Box<Expr>,
        context: Context,
        k: Option<Box<Closure>>,
    },
    IfAfter {
        consequent: Expr,
        alternative: Expr,
        context: Context,
        k: Option<Box<Closure>>,
    },
    Lambda {
        formals: Vec<Expr>,
        body: Vec<Expr>,
        context: Context,
        k: Option<Box<Closure>>,
    },
    Lookup {
        #[serde(skip)]
        r: Arc<Mutex<Env>>,
        context: Context,
        k: Option<Box<Closure>>,
    },
    Set {
        symbol: Box<Expr>,
        e: Box<Expr>,
        context: Context,
        k: Option<Box<Closure>>,
    },
    Return {
        context: Context,
        k: Option<Box<Closure>>,
    },
}

/// Continuation
/// consists of a closure and an argument
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Continuation {
    pub closure: Closure,
    pub arg: Arg,
}
