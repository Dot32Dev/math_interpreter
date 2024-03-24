use std::iter::from_fn;
use std::iter::once;

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
    message: String,
}

impl SyntaxError {
    fn new(message: String) -> Self {
        SyntaxError { message }
    }
}

fn main() -> Result<(), SyntaxError> {
    let tokens = lexer("20 + 32 - 2 + 3.14").unwrap();
    println!("{:?}", tokens);

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

    // tokens.push(Token::EOF);
    Ok(tokens)
}
