/*
TODO

- read
- repl
- program structure (7.1.6) including define

 */
use core::fmt;
use std::collections::HashMap;

#[derive(Clone)]
enum Proc {
    Builtin(String, i8, fn (Vec<Box<Value>>) -> Value),
    Defined(Box<Expr>, Vec<Box<Expr>>),
}

impl fmt::Display for Proc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Proc::Builtin(name, _, _) => format!("built-in procedure '{}'", name),
            Proc::Defined(_, _) => format!("procedure")
        };

        write!(f, "{}", str)

    }
}
#[derive(Clone)]
enum Boolean {
    True,
    False,
}
#[derive(Clone)]
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

#[derive(Clone)]

enum Expr {
    VariableReferenceExpr(String),
    LiteralExpr(Literal),
    ProcedureCallExpr(Box<Expr>, Vec<Box<Expr>>),
    LambdaExpr(Box<Expr>, Vec<Box<Expr>>),
    ConditionalExpr(Box<Expr>, Box<Expr>, Box<Expr>),
    AssignmentExpr(Box<Expr>, Box<Expr>),
    IncludeExpr, // To be implemented
}

#[derive(Clone)]
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
            Value::Proc(p) => format!("{}", p),
            Value::String(s) => format!("\"{}\"", s),
            Value::Symbol(s) => s.clone(),
            Value::Unspecified => String::from("unspecified"),
            Value::Vector => String::from("vector"),        
        };

        write!(f, "{}", str)
    }
}

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
    match env.hash.get(&v) {
        Some(v) => Ok(v.clone()),
        None => Err("variable could not be found"),
    }
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

fn builtin_add(args: Vec<Box<Value>>) -> Value {
    Value::NIL
}

fn init(env: &mut Env) {
    let plus = Proc::Builtin(String::from("+"), -1, builtin_add);
    env.hash.insert("+".to_string(), Value::Proc(plus));   
}

fn read() -> Expr {
    Expr::VariableReferenceExpr(String::from("+"))
}

fn main() {
    let mut global_env = &mut Env {
        hash: HashMap::new(),
        parent: Option::None,
    };

    init(global_env);

    print(eval(read(), &global_env));
}
