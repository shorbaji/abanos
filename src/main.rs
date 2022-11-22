enum expr {
    NIL,
    Number(i64),
    Symbol(String)
}

fn main() {
    let e1: expr = expr::NIL;
    let e2: expr = expr::Number(7);
    let e3: expr = expr::Number(6);
    let e4: expr = expr::Symbol(String::from("+"));

    
    println!("Hello, world!");
}
