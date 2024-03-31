use std::io;

mod error;
use error::SyntaxError;

mod lexer;
use lexer::lexer;
use lexer::Token;

mod parser;
use parser::Parser;

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
    let node = Parser::new(&tokens).parse_expression()?;
    let answer = node.run()?;

    Ok(answer)
}
