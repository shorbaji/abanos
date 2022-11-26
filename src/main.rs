use core::fmt;
use std::collections::HashMap;

enum Proc {
    Builtin,
    Defined(Box<Expr>, Vec<Box<Expr>>),
}

enum Boolean {
    True,
    False,
}
enum Value {
    Boolean(Boolean),
    Bytevector,
    Char(char),
    Eof,
    NIL,
    Number(Number),
    Pair(Box<Value>, Box<Value>),
    Port,
    Proc(Proc),
    String(String),
    Symbol(String),
    Unspecified,
    Vector,
}
enum Expr {
    VariableReferenceExpr(String),
    LiteralExpr(Literal),
    ProcedureCallExpr(Box<Expr>, Vec<Box<Expr>>),
    LambdaExpr(Box<Expr>, Vec<Box<Expr>>),
    ConditionalExpr(Box<Expr>, Box<Expr>, Box<Expr>),
    AssignmentExpr(Box<Expr>, Box<Expr>),
    IncludeExpr, // To be implemented
}

enum Literal {
    Boolean(Boolean),
    Bytevector,
    Char(char),
    Number(Number),
    String(String),
    Vector,
}

struct Env {
    hash: HashMap<String, Value>,
    parent: Option<Box<Env>>,
}

type Location = i64;
type Number = i64;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Value::Boolean(b) => format!("{}", match b {
                Boolean::True => true,
                Boolean::False => false,
            }),
            Value::Bytevector => String::from("bytevector"),
            Value::Char(c) => format!("{}", c),
            Value::Eof => String::from("eof"),            
            Value::NIL => String::from("nil"),
            Value::Number(n) => n.to_string(),
            Value::Pair(a, b) => format!("({} . {})", *a, *b),
            Value::Port => String::from("port"),
            Value::Proc(_) => String::from("proc"),
            Value::String(s) => format!("\"{}\"", s),
            Value::Symbol(s) => s.clone(),
            Value::Unspecified => String::from("unspecified"),
            Value::Vector => String::from("vector"),        
        };

        write!(f, "{}", str)
    }
}

// fn cons(a: Expr, b: Expr) -> Expr {
//     Expr::Pair(Box::new(a), Box::new(b))
// }

// fn car(e: Expr) -> Result<Expr, &'static str> {
//     match e {
//         Expr::Pair(a, _) => Ok(*a),
//         _ => Err("not a pair"),
//     }
// }

// fn cdr(e: Expr) -> Result<Expr, &'static str> {
//     match e {
//         Expr::Pair(_, b) => Ok(*b),
//         _ => Err("not a pair"),
//     }
// }

fn self_eval(l: Literal) -> Value {
    match l {
        Literal::Boolean(b) => Value::Boolean(b),
        Literal::Bytevector => Value::Bytevector,
        Literal::Char(c) => Value::Char(c),
        Literal::Number(n) => Value::Number(n),
        Literal::String(s) => Value::String(s.clone()),
        Literal::Vector => Value::Vector,
    }
}

fn lookup(v: String, env: &Env) -> Result<Value, &'static str>{
    Err("variable could not be found")
}

fn apply(proc: Value, operands: Vec<Box<Value>>) -> Result<Value, &'static str> {
    Ok(Value::NIL)
}

fn evlis(operands: Vec<Box<Expr>>, env: &Env) -> Vec<Box<Value>> {
    vec![]
}

fn is_true(v: Value) -> bool {
    match v {
        Value::Boolean(b) => match b {
            Boolean::True => true,
            Boolean::False => false,
        },
        Value::NIL => false,
        _ => true,
    }
}
fn eval(e: Expr, env: &Env) -> Result<Value, &'static str> {
    match e {
        Expr::LiteralExpr(l) => Ok(self_eval(l)),
        Expr::VariableReferenceExpr(v) => lookup(v, env),
        Expr::ProcedureCallExpr(operator, operands) => {
            match eval(* operator, env) {
                Ok(v) => apply(v, evlis(operands, env)),
                Err(s) => Err(s)
            }
        },
        Expr::LambdaExpr(formals, body)
            =>  Ok(Value::Proc(Proc::Defined(formals, body))),
        Expr::ConditionalExpr(test, consequent, alternate) => {
            match eval(*test, env) {
                Ok(v) => match is_true(v) {
                    true => eval(*consequent, env),
                    false => eval(*alternate, env),
                }
                Err(e) => Err(e)
            }
            
        },
        Expr::AssignmentExpr(var, target) =>
            // need to implement assigment here
            Ok(Value::Unspecified),
        Expr::IncludeExpr => Ok(Value::Unspecified),
    }
}

fn print(r: Result<Value, &'static str>) {
    match r {
        Ok(e) => println!("{}", e),
        Err(s) => println!("{}", s),
    }
}
fn main() {
    let global_env = Env {
        hash: HashMap::new(),
        parent: Option::None,
    };

    let e = Expr::LiteralExpr(Literal::Number(7));
    print(eval(e, &global_env));
}
