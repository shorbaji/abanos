use serde::{Deserialize, Serialize};

use crate::continuation::{Closure, Context, Continuation};
use crate::value::Value;

/// Builtin
/// represents a built-in function (standard procedure)
/// such as `+`, `call/cc`, `read`, etc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Builtin {
    pub name: String,

    /// do not/can not/need not serialize the function pointer
    #[serde(skip, default = "crate::builtin::default")]
    pub f: fn(Vec<Value>, Context, Closure) -> Result<Continuation, String>,
    pub min_args: usize,
    pub max_args: Option<usize>,
}

fn error(_: Vec<Value>, _: Context, _: Closure) -> Result<Continuation, String> {
    Err("default".to_string())
}

fn default() -> fn(Vec<Value>, Context, Closure) -> Result<Continuation, String> {
    error
}
