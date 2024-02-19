use serde::{Deserialize, Serialize};

use crate::continuation::{Closure, Context, Continuation};
use crate::value::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Builtin {
    pub name: String,
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
