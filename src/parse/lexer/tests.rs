#[test]
fn test_number() {
    let s = "1.0";

    let mut lex = crate::parse::lexer::DLexer::new(s);

    assert_eq!(
        lex.next(),
        Some(Ok(crate::parse::lexer::Token::Number(String::from("1.0"))))
    );
}

#[test]
fn test_lexical_errors() {
    let s = "(5%";

    let mut lex = crate::parse::lexer::DLexer::new(s);

    assert_eq!(lex.next(), Some(Ok(crate::parse::lexer::Token::ParenLeft)));

    assert_eq!(lex.next(), Some(Err(())));
}
