use crate::continuation::{Arg, Closure, Context, Continuation};
use crate::value::Value;

pub fn add(args: Vec<Value>, _: Context, k: Closure) -> Result<Continuation, String> {
    if args.is_empty() {
        Err("add: no arguments".to_string())
    } else if !args.iter().all(|v| matches!(v, Value::Number(_))) {
        Err("add: invalid argument".to_string())
    } else {
        let sum = args
            .iter()
            .map(|v| if let Value::Number(n) = v { n } else { "0" })
            .map(str::parse::<i64>)
            .collect::<Result<Vec<i64>, _>>()
            .map_err(|e| e.to_string())?
            .iter()
            .sum::<i64>();

        let value = Value::Number(sum.to_string());
        let arg = Arg::Value(value);
        Ok(Continuation { closure: k, arg })
    }
}
