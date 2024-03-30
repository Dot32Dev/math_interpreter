use crate::error::SyntaxError;
use std::iter::from_fn;
use std::iter::once;

#[derive(Debug)]
pub enum Token {
    Number(f32),
    Variable(String),
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    LeftParen,
    RightParen,
    EOF,
}

pub fn lexer(input: &str) -> Result<Vec<Token>, SyntaxError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(ch) = iter.next() {
        match ch {
            ch if ch.is_whitespace() => continue,
            // Matching numbers
            '0'..='9' | '.' => {
                let n = once(ch)
                    .chain(from_fn(|| {
                        iter.by_ref()
                            .next_if(|s| s.is_ascii_digit() | (s == &'.'))
                    }))
                    .collect::<String>()
                    .parse::<f32>()
                    .unwrap();

                tokens.push(Token::Number(n));
            }
            // Matching variables
            'a'..='z' | 'A'..='Z' => {
                let string_of_letters = once(ch)
                    .chain(from_fn(|| {
                        iter.by_ref().next_if(|c| c.is_ascii_alphabetic())
                    }))
                    .collect::<String>();

                tokens.push(Token::Variable(string_of_letters));
            }
            '+' => tokens.push(Token::Add),
            '-' => tokens.push(Token::Subtract),
            '*' => tokens.push(Token::Multiply),
            '/' => tokens.push(Token::Divide),
            '%' => tokens.push(Token::Modulo),
            '^' => tokens.push(Token::Exponent),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            _ => {
                return Err(SyntaxError::new(format!(
                    "Unrecognised character {}",
                    ch
                )))
            }
        }
    }

    tokens.push(Token::EOF);
    Ok(tokens)
}
