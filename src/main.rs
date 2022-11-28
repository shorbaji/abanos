// TODO
// - read
// - program structure (7.1.6) including define

use core::fmt;
use std::collections::{HashMap, VecDeque};
use std::io::Write;

#[derive(Clone)]
enum Proc {
    Builtin(String, i8, fn (Vec<Box<Value>>) -> Value),
    Defined(Box<Expr>, Vec<Box<Expr>>),
}

impl Proc {
    fn apply(&self, operands: Vec<Box<Value>>) -> Result<Value, &'static str> {
        Ok(Value::NIL)
    }
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

type Number = i64;

#[derive(Clone)]
enum Value {
    Boolean(bool),
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

impl Value {
    fn is_true(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::NIL => false,
            _ => true,
        }
    }

    fn print(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Value::Boolean(b) => format!("{}", *b),
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

impl Expr {    
}

#[derive(Clone)]
enum Literal {
    Boolean(bool),
    Bytevector,
    Char(char),
    Number(Number),
    String(String),
    Vector,
}

impl Literal {
    fn self_eval(&self) -> Value {
        match self {
            Literal::Boolean(b) => Value::Boolean(b.clone()),
            Literal::Bytevector => Value::Bytevector,
            Literal::Char(c) => Value::Char(*c),
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Vector => Value::Vector,
        }
    }
}
struct Env {
    hash: HashMap<String, Value>,
    parent: Option<Box<Env>>,
}

impl Env {
    fn builtin_add(args: Vec<Box<Value>>) -> Value {
        Value::NIL
    }
        
    fn new() -> Self {
        let mut env = Self {
            hash: HashMap::new(),
            parent: Option::None,
        };

        env.init();

        env
    }

    fn init(self: &mut Self) {
        let plus = Proc::Builtin(String::from("+"), -1, Env::builtin_add);
        self.hash.insert("+".to_string(), Value::Proc(plus)); 
    }
 
    fn lookup(&self, v: String) -> Result<Value, &'static str>{
        match self.hash.get(&v) {
            Some(v) => Ok(v.clone()),
            None => Err("variable could not be found"),
        }
    }
    
    fn evlis(&self, operands: Vec<Box<Expr>>) -> Vec<Box<Value>> {
        vec![]
    }
    
    fn eval(&self, e: Expr) -> Result<Value, &'static str> {
        match e {
            Expr::LiteralExpr(l) => Ok(l.self_eval()),
            Expr::VariableReferenceExpr(v) => self.lookup(v),
            Expr::ProcedureCallExpr(operator, operands) => {
                match self.eval(*operator) {
                    Ok(v) => match v { 
                        Value::Proc(p) => p.apply(self.evlis(operands)),
                        _ => Err("not a proc"),
                    }
                    Err(s) => Err(s)
                }
            },
            Expr::LambdaExpr(formals, body) => Ok(Value::Proc(Proc::Defined(formals, body))),
            Expr::ConditionalExpr(test, consequent, alternate) => {
                match self.eval(*test) {
                    Ok(v) => match v.is_true() {
                        true => self.eval(*consequent),
                        false => self.eval(*alternate),
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

}

// read

type Lexeme = String;
struct Lexer {
    buffer: String,
    found: VecDeque<Lexeme>,
    state: i64,
}

impl Lexer {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            state: 0,
            found: VecDeque::new(),
        }
    }
    
    fn next(&mut self) -> Option<Lexeme> {

        if self.found.is_empty() {
            std::io::stdin().read_line(&mut self.buffer).expect("could not read line");

            let mut v: VecDeque<String> = self.buffer.replace("(", " ( ")
                .replace(")", " ) ")
                .split_whitespace()
                .map(|x| x.to_string())
                .collect();
                self.found.append(&mut v);

            self.buffer = String::new();
        }

        if self.found.is_empty() {
            None
        } else {
            self.found.pop_front()
        }
    }
}

struct Reader {
    lexer: Lexer,
}

impl Reader {
    fn new() -> Self {
        Self {
            lexer: Lexer::new(),
        }
    }

    fn read(&mut self) -> Option<Expr> {
        let next = self.lexer.next();
        match next {
            Some(lexeme) => Some(Expr::LiteralExpr(Literal::Number(7))),
            None => None,
        }
    }
    
    fn parse(&mut self, lexeme: Lexeme) -> Option<Expr> {
        None

    }
}


// repl

struct Repl {
    reader: Reader,
    global_env: Env,
}

impl Repl {
    fn new() -> Self {    
        Self {
            reader: Reader::new(),
            global_env: Env::new(),
        }
    }

    fn show_banner(&mut self) -> &mut Self {
        let version = "0.1";

        println!("abanos v{} (c) 2022 Omar Shorbaji", version);
        std::io::stdout().flush();
    
        self
    }
    
        fn go(&mut self) -> Option<String> {
        let mut i: i64 = 0;
        let prompt = "$";
    
        loop {
            print!("{}{} ", i, prompt);
            std::io::stdout().flush();
            match &self.global_env.eval(self.reader.read()?) {
                Ok(s) => println!("{}", s),
                Err(s) => println!("{}", s),
            }
            i = i + 1;
        }
    
        None
    }
    
}

fn main() {
    Repl::new().show_banner().go();
}
