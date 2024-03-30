use crate::error::SyntaxError;
use crate::Token;

#[derive(Debug)]
pub enum Node {
    Number(f32),
    BinaryOp {
        left: Box<Node>,
        op: Operator,
        right: Box<Node>,
    },
}

#[derive(Debug)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
}

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

pub struct Parser<'a, I>
where
    I: Iterator<Item = &'a Token>,
{
    token_iter: std::iter::Peekable<I>,
    // depth: u32,
}

impl<'a> Parser<'a, std::slice::Iter<'a, Token>> {
    pub fn new(input: &'a [Token]) -> Self {
        let iter = input.iter().peekable();
        return Parser {
            token_iter: iter,
            // node: ,
            // depth: 0,
        };
    }

    // This will parse `term +/- term`
    pub fn parse_expression(&mut self) -> Result<Node, SyntaxError> {
        // This will parse and consume all tokens up until a + or -
        // This follows the order of operations, where multiply and division get
        // computed before addition and subtraction
        let mut node = self.parse_term()?;

        // We can't consume the iterator with iter.next(), as then lower
        // functions won't have access to those tokens
        while let Some(&token) = self.token_iter.peek() {
            match token {
                Token::Add | Token::Subtract => {
                    // Consume the +/- token, then parse the next term starting
                    // from the token after the =/-
                    self.token_iter.next();
                    let right_term = self.parse_term()?;
                    // Set the current expressions node to the previously
                    // calculated node, +/- the next term
                    node = Node::BinaryOp {
                        left: Box::new(node),
                        op: token.to_operator()?,
                        right: Box::new(right_term),
                    };
                }
                // Because the expression is the final node that returns, we
                // need to be careful about what exactly it is returning. Only
                // let it break when it is logical to break.
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
    // Technically "number" here could be the result of an expression in
    // brackets () or the result of an exponent (2^5 for example)
    fn parse_term(&mut self) -> Result<Node, SyntaxError> {
        let mut node = self.parse_exponent()?;

        while let Some(&token) = self.token_iter.peek() {
            match token {
                Token::Multiply | Token::Divide | Token::Modulo => {
                    self.token_iter.next();
                    let right_factor = self.parse_exponent()?;
                    node = Node::BinaryOp {
                        left: Box::new(node),
                        op: token.to_operator()?,
                        right: Box::new(right_factor),
                    };
                }
                // If something is wrong, a higher function can figure it out
                _ => {
                    break;
                }
            }
        }

        Ok(node)
    }

    // Exponents have higher precedence than multiply/divide/modulo, so it needs
    // its own function. This was just a copypaste of the term function, where I
    // renamed parse_expression within parse_term to parse_exponent, and changed
    // Token::Multiply | Token::Divide | Token::Modulo to just Token::Exponent
    fn parse_exponent(&mut self) -> Result<Node, SyntaxError> {
        let mut node = self.parse_factor()?;

        while let Some(&token) = self.token_iter.peek() {
            match token {
                Token::Exponent => {
                    self.token_iter.next();
                    let right_factor = self.parse_factor()?;
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

    // Will return either a number or the result of an expression within any
    // brackets it landed on
    fn parse_factor(&mut self) -> Result<Node, SyntaxError> {
        // This function is the only function allowed to consume everything it finds
        match self.token_iter.next() {
            // If it's a number, return the number
            Some(Token::Number(value)) => Ok(Node::Number(*value)),
            Some(Token::LeftParen) => {
                // If we got an opening bracket, parse the expression inside
                let node = self.parse_expression()?;
                // Now after parsing the inner expression, we should get a closing
                // bracket
                match self.token_iter.next() {
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

impl Node {
    // Recursively execurtes the abstract syntax tree! Such beauty.
    pub fn run(self) -> Result<f32, SyntaxError> {
        match self {
            Node::Number(float) => Ok(float),
            #[rustfmt::skip]
            Node::BinaryOp { left, op, right } => match op {
                Operator::Add      => Ok(left.run()? + right.run()?),
                Operator::Subtract => Ok(left.run()? - right.run()?),
                Operator::Multiply => Ok(left.run()? * right.run()?),
                Operator::Divide   => Ok(left.run()? / right.run()?),
                Operator::Modulo   => Ok(left.run()? % right.run()?),
                Operator::Exponent => Ok(left.run()?.powf(right.run()?)),
            },
        }
    }
}
