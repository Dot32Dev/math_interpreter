use std::iter::from_fn;
use std::iter::once;

use std::iter::Peekable;
use std::slice::Iter;

use std::io;

#[derive(Debug)]
enum Token {
    Number(f32),
    Add,
    Subtract,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
    EOF,
}

#[derive(Debug)]
struct SyntaxError {
    // My code doesn't access this field, only Rust does when the main function returns an error.
    #[allow(unused)]
    message: String,
}

impl SyntaxError {
    fn new(message: String) -> Self {
        SyntaxError { message }
    }
}

fn main() -> Result<(), SyntaxError> {
    println!("Enter something to calculate:");

    let mut input_string = String::new();
    io::stdin().read_line(&mut input_string).unwrap();

    let tokens = lexer(input_string.trim())?;

    let node = parse(&tokens)?;

    let answer = run(node)?;

    println!("Answer: {}", answer);
    Ok(())
}

fn lexer(input: &str) -> Result<Vec<Token>, SyntaxError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(ch) = iter.next() {
        match ch {
            ch if ch.is_whitespace() => continue,
            '0'..='9' | '.' => {
                let n: f32 = once(ch)
                    .chain(from_fn(|| {
                        iter.by_ref().next_if(|s| s.is_ascii_digit() | (s == &'.'))
                    }))
                    .collect::<String>()
                    .parse()
                    .unwrap();

                tokens.push(Token::Number(n));
            }
            '+' => tokens.push(Token::Add),
            '-' => tokens.push(Token::Subtract),
            '*' => tokens.push(Token::Multiply),
            '/' => tokens.push(Token::Divide),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            _ => return Err(SyntaxError::new(format!("unrecognised character {}", ch))),
        }
    }

    tokens.push(Token::EOF);
    Ok(tokens)
}

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
}

impl Token {
    fn to_operator(&self) -> Result<Operator, SyntaxError> {
        match self {
            Token::Add => return Ok(Operator::Add),
            Token::Subtract => return Ok(Operator::Subtract),
            Token::Multiply => return Ok(Operator::Multiply),
            Token::Divide => return Ok(Operator::Divide),
            _ => {
                return Err(SyntaxError::new(format!(
                    "Operator expected, got {:?}",
                    self
                )))
            }
        }
    }

    fn to_number(&self) -> Result<f32, SyntaxError> {
        match self {
            Token::Number(float) => return Ok(*float),
            _ => return Err(SyntaxError::new(format!("Number expected, got {:?}", self))),
        }
    }
}

fn parse(input: &[Token]) -> Result<Node, SyntaxError> {
    let mut iter = input.iter().peekable();
    // This is a seperate function because other functions like "parse_factor"
    // can call parse_expression to recursively evaluate expressions inside
    // parenthesis
    parse_expression(&mut iter)
}

// Yes, this function signature is INSANE
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
            Token::EOF => break,
            _ => {
                // This would be a really strange error, and would indicate that
                // parse_term didn't find the entire term
                return Err(SyntaxError::new(format!(
                    "Expected + or -, got {:?}",
                    token
                )));
            }
        }
    }

    Ok(node)
}

fn parse_term<'a>(
    mut iter: &mut std::iter::Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<Node, SyntaxError> {
    match iter.next() {
        Some(token) => {
            match token {
                Token::Number(_) => {
                    let term: Vec<&Token> = once(token)
                        .chain(from_fn(|| {
                            iter.by_ref().next_if(|s| match s {
                                Token::Number(_) => true,
                                Token::Multiply => true,
                                Token::Divide => true,
                                _ => false,
                            })
                        }))
                        .collect();

                    let term_node = match term.len() {
                        n if n % 2 == 1 && n > 1 => {
                            // Only matches if the length is 3, 5, 7, etc (makes room for operators between factors)

                            let left_factor = term[0].to_number()?;
                            let mut node = Node::Number(left_factor);

                            // Iterates two places at a time
                            // for [item_one, item_two] in my_vector.iter().skip(1).chunks(2) {
                            for i in (1..term.len()).step_by(2) {
                                let operator = term[i].to_operator()?;
                                let right_factor = term[i + 1].to_number()?;

                                node = Node::BinaryOp {
                                    left: Box::new(node),
                                    op: operator,
                                    right: Box::new(Node::Number(right_factor)),
                                };
                            }

                            node
                        }
                        1 => {
                            let factor = term[0].to_number()?;
                            Node::Number(factor)
                        }
                        _ => {
                            return Err(SyntaxError::new("Malformed expression".to_string()));
                        }
                    };
                    Ok(term_node)
                }
                _ => {
                    return Err(SyntaxError::new(
                        "Why does the term begin with something other than a number".to_string(),
                    ))
                }
            }
        }
        None => return Err(SyntaxError::new("There is nothing to parse".to_string())),
    }
}

fn parse_factor<'a>(
    mut iter: &mut std::iter::Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<Node, SyntaxError> {
    match iter.next() {
        // If it's a number, return the number
        Some(Token::Number(value)) => Ok(Node::Number(*value)),
        Some(Token::LeftParen) => {
            // If we got an opening bracket, parse the expression inside
            let node = parse_expression(&mut iter)?;
            // Now after parsing the inner expression, we should get a closing bracket
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
        Some(token) => Err(SyntaxError::new(format!(
            "Expected number or opening bracket, got {:?}",
            token
        ))),
        None => Err(SyntaxError::new(
            "Expected number or opening bracket, got nothing".to_string(),
        )),
    }
}

fn run(node: Node) -> Result<f32, SyntaxError> {
    match node {
        Node::Number(float) => return Ok(float),
        Node::BinaryOp { left, op, right } => match op {
            Operator::Add => {
                return Ok(run(*left)? + run(*right)?);
            }
            Operator::Subtract => {
                return Ok(run(*left)? - run(*right)?);
            }
            Operator::Multiply => {
                return Ok(run(*left)? * run(*right)?);
            }
            Operator::Divide => {
                return Ok(run(*left)? / run(*right)?);
            }
        },
    }
}
