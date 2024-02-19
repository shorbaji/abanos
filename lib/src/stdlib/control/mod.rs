use crate::continuation::{Arg, Closure, Context, Continuation};
use crate::value::Value;

pub fn call_cc(args: Vec<Value>, context: Context, k: Closure) -> Result<Continuation, String> {
    if args.len() != 1 {
        return Err("call/cc expects 1 argument".to_string());
    }

    // we should have a procedure in the first argument
    // we have to return a continuation
    // this will be an apply continuation
    // operator is the first argument
    // operands is a list of the context k

    let operator = Box::new(args[0].clone());
    let operands = vec![Value::Continuation(k.clone())];

    let closure = Closure::Call {
        operator,
        context,
        k: Some(Box::new(k)),
    };
    let arg = Arg::ValueList(operands);

    Ok(Continuation { closure, arg })
}
