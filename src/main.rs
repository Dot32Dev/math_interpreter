use std::io;

mod error;
use error::SyntaxError;

mod lexer;
use lexer::lexer;
use lexer::Token;

#[derive(Debug)]
enum Node {
    Number(f32),
    BinaryOp {
        left: Box<Node>,
        op: Operator,
        right: Box<Node>,
    },
}

#[derive(Debug)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
}

// #[derive(Debug)]
// enum Token {
//     Number(f32),
//     Variable(String),
//     Add,
//     Subtract,
//     Multiply,
//     Divide,
//     Modulo,
//     Exponent,
//     LeftParen,
//     RightParen,
//     EOF,
// }

impl Token {
    fn to_operator(&self) -> Result<Operator, SyntaxError> {
        match self {
            Token::Add => Ok(Operator::Add),
            Token::Subtract => Ok(Operator::Subtract),
            Token::Multiply => Ok(Operator::Multiply),
            Token::Divide => Ok(Operator::Divide),
            Token::Modulo => Ok(Operator::Modulo),
            Token::Exponent => Ok(Operator::Exponent),
            _ => Err(SyntaxError::new(format!(
                "Operator expected, got {:?}",
                self
            ))),
        }
    }
}

// #[derive(Debug)]
// struct SyntaxError {
//     message: String,
// }

// impl SyntaxError {
//     fn new(message: String) -> Self {
//         SyntaxError { message }
//     }
// }

fn main() {
    println!("Enter something to calculate:");

    let mut input_string = String::new();
    io::stdin().read_line(&mut input_string).unwrap();

    match sandbox(input_string.trim()) {
        Ok(answer) => {
            println!("Answer: {}", answer);
        }
        Err(syntax_error) => {
            println!("Error: {}", syntax_error.message);
        }
    }
}

fn sandbox(input: &str) -> Result<f32, SyntaxError> {
    let tokens = lexer(input)?;

    let node = parse(&tokens)?;

    let answer = run(node)?;

    Ok(answer)
}

// fn lexer(input: &str) -> Result<Vec<Token>, SyntaxError> {
//     let mut tokens: Vec<Token> = Vec::new();
//     let mut iter = input.chars().peekable();

//     while let Some(ch) = iter.next() {
//         match ch {
//             ch if ch.is_whitespace() => continue,
//             // Matching numbers
//             '0'..='9' | '.' => {
//                 let n = once(ch)
//                     .chain(from_fn(|| {
//                         iter.by_ref().next_if(|s| s.is_ascii_digit() | (s == &'.'))
//                     }))
//                     .collect::<String>()
//                     .parse::<f32>()
//                     .unwrap();

//                 tokens.push(Token::Number(n));
//             }
//             // Matching variables
//             'a'..='z' | 'A'..='Z' => {
//                 let string_of_letters = once(ch)
//                     .chain(from_fn(|| {
//                         iter.by_ref().next_if(|c| c.is_ascii_alphabetic())
//                     }))
//                     .collect::<String>();

//                 tokens.push(Token::Variable(string_of_letters));
//             }
//             '+' => tokens.push(Token::Add),
//             '-' => tokens.push(Token::Subtract),
//             '*' => tokens.push(Token::Multiply),
//             '/' => tokens.push(Token::Divide),
//             '%' => tokens.push(Token::Modulo),
//             '^' => tokens.push(Token::Exponent),
//             '(' => tokens.push(Token::LeftParen),
//             ')' => tokens.push(Token::RightParen),
//             _ => return Err(SyntaxError::new(format!("Unrecognised character {}", ch))),
//         }
//     }

//     tokens.push(Token::EOF);
//     Ok(tokens)
// }

// A recursive decent parser
fn parse(input: &[Token]) -> Result<Node, SyntaxError> {
    let mut iter = input.iter().peekable();
    // This is a seperate function because other functions like "parse_factor"
    // can call parse_expression to recursively evaluate expressions inside
    // parenthesis
    parse_expression(&mut iter)
}

// Yes, this function signature is INSANE
// This will parse `term +/- term`
fn parse_expression<'a>(
    mut iter: &mut std::iter::Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<Node, SyntaxError> {
    // This will parse and consume all tokens up until a + or -
    // This follows the order of operations, where multiply and division get
    // computed before addition and subtraction
    let mut node = parse_term(&mut iter)?;

    // We can't consume the iterator with iter.next(), as then lower functions
    // won't have access to those tokens
    while let Some(&token) = iter.peek() {
        match token {
            Token::Add | Token::Subtract => {
                // Consume the +/- token, then parse the next term starting from
                // the token after the =/-
                iter.next();
                let right_term = parse_term(&mut iter)?;
                // Set the current expressions node to the previously calculated
                // node, +/- the next term
                node = Node::BinaryOp {
                    left: Box::new(node),
                    op: token.to_operator()?,
                    right: Box::new(right_term),
                };
            }
            // Because the expression is the final node that returns, we need to
            // be careful about what exactly it is returning. Only let it break
            // when it is logical to break.
            Token::EOF | Token::RightParen => break,
            token => {
                return Err(SyntaxError::new(format!(
                    "Expected operator or end of file, got {:?}",
                    token
                )))
            }
        }
    }

    Ok(node)
}

// This should parse `number multiply/divide/modulo number`
// Technically "number" here could be the result of an expression in brackets ()
// or the result of an exponent (2^5 for example)
fn parse_term<'a>(
    mut iter: &mut std::iter::Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<Node, SyntaxError> {
    let mut node = parse_exponent(&mut iter)?;

    while let Some(&token) = iter.peek() {
        match token {
            Token::Multiply | Token::Divide | Token::Modulo => {
                iter.next();
                let right_factor = parse_exponent(&mut iter)?;
                node = Node::BinaryOp {
                    left: Box::new(node),
                    op: token.to_operator()?,
                    right: Box::new(right_factor),
                };
            }
            // If something is wrong, a higher function will probably understand
            _ => {
                break;
            }
        }
    }

    Ok(node)
}

// Exponents have higher precedence than multiply/divide/modulo, so it needs its
// own function. This was just a copypaste of the term function, where I renamed
// parse_expression in the term function to parse_exponent, and changed
// Token::Multiply | Token::Divide | Token::Modulo to just Token::Exponent
fn parse_exponent<'a>(
    mut iter: &mut std::iter::Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<Node, SyntaxError> {
    let mut node = parse_factor(&mut iter)?;

    while let Some(&token) = iter.peek() {
        match token {
            Token::Exponent => {
                iter.next();
                let right_factor = parse_factor(&mut iter)?;
                node = Node::BinaryOp {
                    left: Box::new(node),
                    op: token.to_operator()?,
                    right: Box::new(right_factor),
                };
            }
            _ => {
                break;
            }
        }
    }

    Ok(node)
}

// Will return either a number or an expression within any brackets it lands on
fn parse_factor<'a>(
    mut iter: &mut std::iter::Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<Node, SyntaxError> {
    // This function is the only function allowed to consume everything it finds
    match iter.next() {
        // If it's a number, return the number
        Some(Token::Number(value)) => Ok(Node::Number(*value)),
        Some(Token::LeftParen) => {
            // If we got an opening bracket, parse the expression inside
            let node = parse_expression(&mut iter)?;
            // Now after parsing the inner expression, we should get a closing
            // bracket
            match iter.next() {
                // This will consume the closing bracket and return node
                Some(Token::RightParen) => Ok(node),
                Some(token) => Err(SyntaxError::new(format!(
                    "Expected closing bracket, got {:?}",
                    token
                ))),
                _ => Err(SyntaxError::new(
                    "Expected closing bracket, got nothing".to_string(),
                )),
            }
        }
        Some(Token::Variable(var_name)) => Ok(Node::Number(evaluate_variable(var_name)?)),
        Some(token) => Err(SyntaxError::new(format!(
            "Expected number or opening bracket, got {:?}",
            token
        ))),
        None => Err(SyntaxError::new(
            "Expected number or opening bracket, got nothing".to_string(),
        )),
    }
}

fn evaluate_variable(var_name: &String) -> Result<f32, SyntaxError> {
    match &var_name[..] {
        "pi" => Ok(std::f32::consts::PI),
        string => Err(SyntaxError::new(format!(
            "Unidentified variable {:?}",
            string
        ))),
    }
}

// Recursively execurtes the abstract syntax tree! Such beauty.
fn run(node: Node) -> Result<f32, SyntaxError> {
    match node {
        Node::Number(float) => Ok(float),
        #[rustfmt::skip]
        Node::BinaryOp { left, op, right } => match op {
            Operator::Add      => Ok(run(*left)? + run(*right)?),
            Operator::Subtract => Ok(run(*left)? - run(*right)?),
            Operator::Multiply => Ok(run(*left)? * run(*right)?),
            Operator::Divide   => Ok(run(*left)? / run(*right)?),
            Operator::Modulo   => Ok(run(*left)? % run(*right)?),
            Operator::Exponent => Ok(run(*left)?.powf(run(*right)?)),
        },
    }
}
