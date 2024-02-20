use crate::parse::{error::ParseError, Expr, Parser, lexer::Token};

fn test_meta(s: &str, expected: Vec<Result<Expr, ParseError>>) {
    let reader = Parser::new(s.as_bytes());

    for (expr, x) in reader.zip(expected) {
        match (expr, x) {
            (Ok(e), Ok(x)) => assert_eq!(e, x),
            (Err(e), Err(x)) => assert_eq!(e, x),
            (a, b) => panic!("expected {:?}, got {:?}", b, a),
        }
    }
}

#[test]
fn test_boolean() {
    let s = "#t #true #f #false";
    let expected = vec![
        Ok(Expr::Boolean(true)),
        Ok(Expr::Boolean(true)),
        Ok(Expr::Boolean(false)),
        Ok(Expr::Boolean(false)),
    ];

    test_meta(s, expected);
}

#[test]
fn test_bytevector() {
    let s = "
    #u8(1 2 3)
    #u8(1 512 3)
    ";

    let expected = vec![
        Ok(Expr::Bytevector(vec![1, 2, 3])),
        Err(ParseError::UnexpectedToken(Token::Number("512".to_string()), 1)),
    ];

    test_meta(s, expected);
}

#[test]
fn test_char() {
    let s = "#\\a #\\newline #\\space #\\tab";

    let expected = vec![
        Ok(Expr::Char('a')),
        Ok(Expr::Char('\n')),
        Ok(Expr::Char(' ')),
        Ok(Expr::Char('\t')),
    ];

    test_meta(s, expected);
}

#[test]
fn test_number() {
    let s = "1 1.0 1/2 1+2i 1.0+2.0i 1/2+3/4i";
    let expected = vec![
        Ok(Expr::Number(String::from("1"))),
        Ok(Expr::Number(String::from("1.0"))),
        Ok(Expr::Number(String::from("1/2"))),
        Ok(Expr::Number(String::from("1+2i"))),
        Ok(Expr::Number(String::from("1.0+2.0i"))),
        Ok(Expr::Number(String::from("1/2+3/4i"))),
    ];
    test_meta(s, expected)
}

#[test]
fn test_string() {
    let s = "\"hello\" \"world\"";
    let expected = vec![
        Ok(Expr::String(String::from("hello"))),
        Ok(Expr::String(String::from("world"))),
    ];

    test_meta(s, expected);
}

#[test]
fn test_vector() {
    let s = "#(1 2 3)";
    let expected = vec![Ok(Expr::Vector(vec![
        Expr::Number(String::from("1")),
        Expr::Number(String::from("2")),
        Expr::Number(String::from("3")),
    ]))];

    test_meta(s, expected);
}

#[test]
fn test_quote() {
    let s = "'1\n(quote 1)\n";
    let expected = vec![
        Ok(Expr::Number(String::from("1"))),
        Ok(Expr::Number(String::from("1"))),
    ];

    test_meta(s, expected);
}

#[test]
fn test_assignment() {
    let s = "(set! x 1)\n(set! double (lambda (x) (* x 2)))";

    let expected = vec![
        Ok(Expr::Set(
            Box::new(Expr::Variable(String::from("x"))),
            Box::new(Expr::Number(String::from("1"))),
        )),
        Ok(Expr::Set(
            Box::new(Expr::Variable(String::from("double"))),
            Box::new(Expr::Lambda(
                vec![Expr::Variable(String::from("x"))],
                vec![Expr::Apply(
                    Box::new(Expr::Variable(String::from("*"))),
                    vec![
                        Expr::Variable(String::from("x")),
                        Expr::Number(String::from("2")),
                    ],
                )],
            )),
        )),
    ];

    test_meta(s, expected);
}

#[test]
fn test_definition() {
    let s = "(define x 1)\n(define (double x) (* x 2))";

    let expected = vec![
        Ok(Expr::Define(
            Box::new(Expr::Variable(String::from("x"))),
            Box::new(Expr::Number(String::from("1"))),
        )),
        Ok(Expr::Define(
            Box::new(Expr::Variable(String::from("double"))),
            Box::new(Expr::Lambda(
                vec![Expr::Variable(String::from("x"))],
                vec![Expr::Apply(
                    Box::new(Expr::Variable(String::from("*"))),
                    vec![
                        Expr::Variable(String::from("x")),
                        Expr::Number(String::from("2")),
                    ],
                )],
            )),
        )),
    ];

    test_meta(s, expected);
}

#[test]
fn test_conditional() {
    let s = "(if #t 1 2)\n (if #f 1 2 3 4)";

    let expected = vec![
        Ok(Expr::If(
            Box::new(Expr::Boolean(true)),
            Box::new(Expr::Number(String::from("1"))),
            Box::new(Expr::Number(String::from("2"))),
        )),
        Err(ParseError::UnexpectedToken(Token::Number("3".to_string()), 1)),
    ];

    test_meta(s, expected);
}

#[test]
fn test_lambda() {
    let s = "(lambda (x) x)\n(lambda (x y) (+ x y))";

    let expected = vec![
        Ok(Expr::Lambda(
            vec![Expr::Variable(String::from("x"))],
            vec![Expr::Variable(String::from("x"))],
        )),
        Ok(Expr::Lambda(
            vec![
                Expr::Variable(String::from("x")),
                Expr::Variable(String::from("y")),
            ],
            vec![Expr::Apply(
                Box::new(Expr::Variable(String::from("+"))),
                vec![
                    Expr::Variable(String::from("x")),
                    Expr::Variable(String::from("y")),
                ],
            )],
        )),
    ];

    test_meta(s, expected);
}

#[test]
fn test_application() {
    let s = "(+ 1 2)\n(+ 1 2 3)\n ((foo) bar baz)";

    let expected = vec![
        Ok(Expr::Apply(
            Box::new(Expr::Variable(String::from("+"))),
            vec![
                Expr::Number(String::from("1")),
                Expr::Number(String::from("2")),
            ],
        )),
        Ok(Expr::Apply(
            Box::new(Expr::Variable(String::from("+"))),
            vec![
                Expr::Number(String::from("1")),
                Expr::Number(String::from("2")),
                Expr::Number(String::from("3")),
            ],
        )),
        Ok(Expr::Apply(
            Box::new(Expr::Apply(
                Box::new(Expr::Variable(String::from("foo"))),
                vec![],
            )),
            vec![
                Expr::Variable(String::from("bar")),
                Expr::Variable(String::from("baz")),
            ],
        )),
    ];

    test_meta(s, expected);
}

#[test]
fn test_variable() {
    let s = "x\ny\nz";

    let expected = vec![
        Ok(Expr::Variable(String::from("x"))),
        Ok(Expr::Variable(String::from("y"))),
        Ok(Expr::Variable(String::from("z"))),
    ];

    test_meta(s, expected);
}

#[test]
fn test_recover() {
    let s = "(if 1) 1 ( ( ( ( if )))) 7";

    let expected = vec![
        Err(ParseError::UnexpectedToken(Token::ParenRight, 1)),
        Ok(Expr::Number(String::from("1"))),
        Err(ParseError::UnexpectedToken(Token::ParenRight, 4)),
        Ok(Expr::Number(String::from("7"))),
    ];

    test_meta(s, expected);
}
