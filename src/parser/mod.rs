use crate::lexer::{Token, lexer};

#[derive(Debug, PartialEq)]
enum Statement {
    Let{ name: String, value: Expr},
    Return{ value: Expr },
    Expression(Expr),
}

#[derive(Debug, PartialEq)]
enum Expr {
    Const(u32),
    Boolean(bool),
    Ident(String),
    Prefix{prefix: Prefix, value: Box<Expr>},
    Infix{left: Box<Expr>, operator: Operator, right: Box<Expr>},
    If{condition: Box<Expr>, consequence: Vec<Statement>, alternative: Vec<Statement>},
}

#[derive(Debug, PartialEq)]
enum Prefix {
    Bang,
    Minus,
}

#[derive(Debug, PartialEq)]
enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
}

#[derive(PartialOrd, PartialEq)]
enum Precedence {
    Lowest,
    Equals ,     // ==
    LessGreater, // > or <
    Sum,         // +
    Product,     // *
    Prefix,      // -X or !X
    Call,        // myFunction(X)
}

fn parse(input: &mut Vec<Token>) -> Vec<Statement> {
    let mut program = vec![];

    loop {
        let token = &input[0];

        match token {
            Token::EOF => break,
            Token::LET => parse_let(input, &mut program),
            Token::RETURN => parse_return(input, &mut program),
            Token::RBRACE => {
                break;
            },
            _ => program.push(
                Statement::Expression(
                    parse_expression(input, Precedence::Lowest)
                )
            )
        }
        assert_eq!(Token::SEMICOLON, input.remove(0));

    }

    program
}

fn parse_let(input: &mut Vec<Token>, program: &mut Vec<Statement>) {
    assert_eq!(Token::LET, input.remove(0));
    let name = match input.remove(0) {
        Token::IDENT(name) => name,
        _ => panic!("parse error at let statement"),
    };
    assert_eq!(Token::ASSIGN, input.remove(0));
    let value = parse_expression(input, Precedence::Lowest);
    program.push(Statement::Let {name, value});
}

fn parse_return(input: &mut Vec<Token>, program: &mut Vec<Statement>) {
    assert_eq!(Token::RETURN, input.remove(0));
    let value = match input.remove(0) {
        Token::INT(value) => value,
        _ => panic!("parse error at return statement"),
    };
    program.push(Statement::Return {value: Expr::Const(value)});
}

fn parse_expression(input: &mut Vec<Token>, precedence: Precedence) -> Expr {
    let mut left_expr = match input.remove(0) {
        Token::INT(value) => Expr::Const(value),
        Token::TRUE => Expr::Boolean(true),
        Token::FALSE => Expr::Boolean(false),
        Token::IDENT(value) => Expr::Ident(value),
        Token::BANG => Expr::Prefix{
            prefix: Prefix::Bang,
            value: Box::new(parse_expression(input, Precedence::Prefix))
        },
        Token::MINUS => Expr::Prefix{
            prefix: Prefix::Minus,
            value: Box::new(parse_expression(input, Precedence::Prefix))
        },
        Token::LPAREN => {
            let expr = parse_expression(input, Precedence::Lowest);
            assert_eq!(Token::RPAREN, input.remove(0));

            expr
        },
        Token::IF => {
            assert_eq!(Token::LPAREN, input.remove(0));
            let condition = parse_expression(input, Precedence::Lowest);
            assert_eq!(Token::RPAREN, input.remove(0));

            assert_eq!(Token::LBRACE, input.remove(0));
            let consequence = parse(input);
            assert_eq!(Token::RBRACE, input.remove(0));

            let alternative = if &input[0] == &Token::ELSE {
                input.remove(0);

                assert_eq!(Token::LBRACE, input.remove(0));
                let alternative = parse(input);
                assert_eq!(Token::RBRACE, input.remove(0));

                alternative
            } else {
                Vec::new()
            };

            Expr::If {
                condition: Box::new(condition),
                consequence,
                alternative,
            }
        }
        _ => panic!("parse error at expression"),
    };

    let mut next_token = &input[0];
    while precedence < next_token.precedence() {
        left_expr = parse_infix(left_expr, input);
        next_token = &input[0];
    }

    left_expr
}

fn parse_infix(left: Expr, input: &mut Vec<Token>) -> Expr {
    let next_token = input.remove(0);
    let operator = match &next_token {
        Token::PLUS => Operator::Plus,
        Token::MINUS => Operator::Minus,
        Token::SLASH => Operator::Divide,
        Token::ASTERISK => Operator::Multiply,
        Token::LT => Operator::LessThan,
        Token::GT => Operator::GreaterThan,
        Token::EQ => Operator::Equals,
        Token::NOT_EQ => Operator::NotEquals,
        _ => panic!("parse infix called on invalid operator"),
    };
    Expr::Infix {
        left: Box::new(left),
        operator,
        right: Box::new(parse_expression(input, next_token.precedence())),
    }
}

impl Token {
    fn precedence(&self) -> Precedence {
        match self {
            Token::PLUS => Precedence::Sum,
            Token::MINUS => Precedence::Sum,
            Token::SLASH => Precedence::Product,
            Token::ASTERISK => Precedence::Product,
            Token::LT => Precedence::LessGreater,
            Token::GT => Precedence::LessGreater,
            Token::EQ => Precedence::Equals,
            Token::NOT_EQ => Precedence::Equals,
            _ => Precedence::Lowest
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_let() {
        let input = "let x = 5;";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Let { name: String::from("x"), value: Expr::Const(5) },
            ],
            ast
        );
    }

    #[test]
    fn parse_return() {
        let input = "return 5;";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Return { value: Expr::Const(5) },
            ],
            ast
        );
    }

    #[test]
    fn parse_let_ident() {
        let input = "let myVar = anotherV;";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Let { name: String::from("myVar"), value: Expr::Ident(String::from("anotherV")) },
            ],
            ast
        );
    }

    #[test]
    fn parse_expression_statement() {
        let input = "foo;";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Expression(Expr::Ident(String::from("foo"))),
            ],
            ast
        );
    }

    #[test]
    fn parse_expression_statement_const() {
        let input = "5;";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Expression(Expr::Const(5)),
            ],
            ast
        );
    }

    #[test]
    fn parse_prefix_expression() {
        let input = "!5; -15;";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Expression(
                    Expr::Prefix{
                        prefix: Prefix::Bang,
                        value: Box::new(Expr::Const(5))
                    }
                ),
                Statement::Expression(
                    Expr::Prefix{
                        prefix: Prefix::Minus,
                        value: Box::new(Expr::Const(15))
                    }
                ),
            ],
            ast
        );
    }

    #[test]
    fn precedence() {
        assert!(Precedence::Lowest < Precedence::Call);
    }

    #[test]
    fn parse_infix_expressions() {
        let test_cases = vec![
            ("5 + 6;", Operator::Plus),
            ("5 - 6;", Operator::Minus),
            ("5 * 6;", Operator::Multiply),
            ("5 / 6;", Operator::Divide),
            ("5 > 6;", Operator::GreaterThan),
            ("5 < 6;", Operator::LessThan),
            ("5 == 6;", Operator::Equals),
            ("5 != 6;", Operator::NotEquals),
        ];

        for (input, operator) in test_cases {
            parse_infix_expression(input, operator);
        }
    }

    fn parse_infix_expression(input: &str, operator: Operator) {
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Expression(Expr::Infix{
                    left: Box::new(Expr::Const(5)),
                    operator,
                    right: Box::new(Expr::Const(6)),
                }),
            ],
            ast
        );
    }

    #[test]
    fn parse_infix_expression_order_of_operations() {
        let input = "-a * 6;";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Expression(Expr::Infix{
                    left: Box::new(Expr::Prefix{ prefix: Prefix::Minus, value: Box::new(Expr::Ident(String::from("a")))}),
                    operator: Operator::Multiply,
                    right: Box::new(Expr::Const(6)),
                }),
            ],
            ast
        );
    }

    #[test]
    fn parse_bool() {
        let input = "!true == false;";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Expression(Expr::Infix{
                    left: Box::new(Expr::Prefix{ prefix: Prefix::Bang, value: Box::new(Expr::Boolean(true))}),
                    operator: Operator::Equals,
                    right: Box::new(Expr::Boolean(false)),
                }),
            ],
            ast
        );
    }

    #[test]
    fn parse_paren() {
        let input = "1 + (2 + 3);";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Expression(Expr::Infix{
                    left: Box::new(Expr::Const(1)),
                    operator: Operator::Plus,
                    right: Box::new(Expr::Infix {
                        left: Box::new(Expr::Const(2)),
                        operator: Operator::Plus,
                        right: Box::new(Expr::Const(3))
                    }),
                }),
            ],
            ast
        );
    }

    #[test]
    fn parse_if() {
        let input = "if (5) { 6; };";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Expression(Expr::If{
                    condition: Box::new(Expr::Const(5)),
                    consequence: vec![Statement::Expression(Expr::Const(6))],
                    alternative: Vec::new()
                }),
            ],
            ast
        );
    }

    #[test]
    fn parse_if_else() {
        let input = "if (5) { 6; } else { 7; };";
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);

        assert_eq!(
            vec![
                Statement::Expression(Expr::If{
                    condition: Box::new(Expr::Const(5)),
                    consequence: vec![Statement::Expression(Expr::Const(6))],
                    alternative: vec![Statement::Expression(Expr::Const(7))],
                }),
            ],
            ast
        );
    }

}
