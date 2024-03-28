use std::iter::from_fn;
use std::iter::once;

use std::iter::Peekable;
use std::slice::Iter;

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
    let tokens = lexer("2 * 3 + 4 * 5")?;
    println!("{:?}", tokens);

    parse(&tokens)?;

    let mut answer = 0.0;
    let mut last_operand = Token::Add;
    for token in tokens {
        match token {
            Token::Number(int) => match last_operand {
                Token::Add => {
                    answer += int;
                }
                Token::Subtract => {
                    answer -= int;
                }
                _ => (),
            },
            Token::Add | Token::Subtract => {
                last_operand = token;
            }
            Token::EOF => break,
            _ => {
                return Err(SyntaxError::new(format!(
                    "{:?} is currently unsupported",
                    token
                )))
            }
        }
    }

    println!("Answer: {}", answer);
    Ok(())
}

fn lexer(input: &str) -> Result<Vec<Token>, SyntaxError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(ch) = iter.next() {
        match ch {
            ch if ch.is_whitespace() => continue,
            '1'..='9' | '.' => {
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
            _ => return Err(SyntaxError::new(format!("unrecognized character {}", ch))),
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

    let mut node = parse_term(&mut iter)?;

    while let Some(&token) = iter.peek() {
        match token {
            Token::Add | Token::Subtract => {
                iter.next();
                let right_term = parse_term(&mut iter)?;
                // node = Node::BinaryOp(Box::new(node), token.into(), Box::new(right_expr));
                node = Node::BinaryOp {
                    left: Box::new(node),
                    op: token.to_operator()?,
                    right: Box::new(right_term),
                };
            }
            Token::EOF => break,
            _ => {
                return Err(SyntaxError::new(
                    "Bro I don't know what you did to cause this".to_string(),
                ))
            }
        }
    }

    println!("{:#?}", node);

    Ok(node)
}

// WHAT IS THIS FUNCTION SIGNATURE, had to ask ChatGPT for this shit
fn parse_term<'a>(
    iter: &mut std::iter::Peekable<impl Iterator<Item = &'a Token>>,
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
