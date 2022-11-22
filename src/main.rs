use core::fmt;

enum Proc {
    Builtin,
    Defined,
}

enum Value {
    NIL,
    Proc(Proc),
    Number(i64),
    String(String),
    Char(char),
}
enum Expr {
    NIL,
    Number(i64),
    Symbol(String),
    Char(char),
    Pair(Box<Expr>, Box<Expr>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Value::NIL => String::from("nil"),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Char(c) => format!("{}", c),
            Value::Proc(_) => String::from("proc"),
        };

        write!(f, "{}", str)
    }
}

fn cons(a: Expr, b: Expr) -> Expr {
    Expr::Pair(Box::new(a), Box::new(b))
}

fn car(e: Expr) -> Result<Expr, &'static str> {
    match e {
        Expr::Pair(a, _) => Ok(*a),
        _ => Err("not a pair"),
    }
}

fn cdr(e: Expr) -> Result<Expr, &'static str> {
    match e {
        Expr::Pair(_, b) => Ok(*b),
        _ => Err("not a pair"),
    }
}

fn eval(e: Expr) -> Result<Value, &'static str> {
    match e {
        Expr::NIL => Ok(Value::NIL),
        Expr::Number(n) => Ok(Value::Number(n)),
        Expr::Char(c) => Ok(Value::Char(c)),
        Expr::Symbol(s) => Err("symbol not found"),
        Expr::Pair(_, _) => Err("Don't know how to deal with that"),

    }
}

fn print(r: Result<Value, &'static str>) {
    match r {
        Ok(e) => println!("{}", e),
        Err(s) => println!("{}", s),
    }
}
fn main() {
    let e1: Expr = Expr::NIL;
    let e2: Expr = Expr::Number(7);
    let e3: Expr = Expr::Number(6);
    let e4: Expr = Expr::Symbol(String::from("+"));
    let e5: Expr = cons(e2,
                        cons(e3, e1));

    print(eval(e4));
    print(eval(e5));
}
