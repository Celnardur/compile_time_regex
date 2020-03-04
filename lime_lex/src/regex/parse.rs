use super::simplify::Token;
use crate::Error;
use std::rc::Rc;
use BinaryOperation::*;
use UnaryOperation::*;

type Pointer = Rc<RAST>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinaryOperation {
    Concat,
    Alternation,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UnaryOperation {
    MinMax(u8, u8),
    Times(u8),
    KleenClosure,
    Question,
    Plus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RAST {
    Binary(Pointer, Pointer, BinaryOperation),
    Unary(Pointer, UnaryOperation),
    Atomic(u8),
}

pub fn parse(regex: &[Token]) -> Result<RAST, Error> {
    let mut regex: Vec<Token> = regex.iter().cloned().rev().collect();
    let rast = parse_regex(&mut regex)?;
    if !regex.is_empty() {
        return Err(Error::new("Regex stoped parsing before the end"));
    }
    Ok(rast)
}

pub fn parse_regex(regex: &mut Vec<Token>) -> Result<RAST, Error> {
    parse_binary(regex)
}

fn parse_binary(regex: &mut Vec<Token>) -> Result<RAST, Error> {
    let unary = parse_unary(regex)?;
    if let Some(prime) = parse_binary_prime(regex)? {
        Ok(RAST::Binary(Rc::new(unary), Rc::new(prime.0), prime.1))
    } else {
        Ok(unary)
    }
}

fn parse_binary_prime(regex: &mut Vec<Token>) -> Result<Option<(RAST, BinaryOperation)>, Error> {
    if let Some(t) = regex.pop() {
        let token = match t {
            Token::Concat => Concat,
            Token::Alternation => Alternation,
            _ => {
                regex.push(t);
                return Ok(None);
            }
        };
        let unary = parse_unary(regex)?;
        if let Some(prime) = parse_binary_prime(regex)? {
            Ok(Some((RAST::Binary(Rc::new(unary), Rc::new(prime.0), prime.1), token)))
        } else {
            Ok(Some((unary, token)))
        }
    } else {
        Ok(None)
    }
}

fn parse_unary(regex: &mut Vec<Token>) -> Result<RAST, Error> {
    let group = parse_group(regex)?;
    let ops = parse_unary_prime(regex)?;
    if ops.is_empty() {
        return Ok(group);
    }
    
    let mut rast = group;
    for op in ops.iter().rev() {
        rast = RAST::Unary(Rc::new(rast), *op);
    }
    Ok(rast)
}

fn parse_unary_prime(regex: &mut Vec<Token>) -> Result<Vec<UnaryOperation>, Error> {
    if let Some(t) = regex.pop() {
        let token = match t {
            Token::KleenClosure => KleenClosure,
            Token::Question => Question,
            Token::Plus => Plus,
            Token::Times(min) => Times(min),
            Token::MinMax(min, max) => MinMax(min, max),
            _ => {
                regex.push(t);
                return Ok(Vec::new());
            },
        };
        let mut ops = parse_unary_prime(regex)?;
        ops.push(token);

        Ok(ops)
    } else {
        Ok(Vec::new())
    }
}
    
fn parse_group(regex: &mut Vec<Token>) -> Result<RAST, Error> {
    if let Some(t) = regex.pop() {
        match t {
            Token::Character(c) => Ok(RAST::Atomic(c)),
            Token::LParen => {
                let group = parse_regex(regex)?;
                if let Some(t) = regex.pop() {
                    match t {
                        Token::RParen => Ok(group),
                        _ => Err(Error::new("Unexpected token, expected ')'"))
                    }
                } else {
                    Err(Error::new("Reached end of regex while parsing"))
                }
            }, 
            _ => Err(Error::new("Unexpected token, expected char or '('")),
        }
    } else {
        Err(Error::new("Reached end of regex while parsing"))
    }
}
