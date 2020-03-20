use super::simplify::Token;
use crate::Error;
use BinaryOperation::*;
use UnaryOperation::*;

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
    Binary(Box<RAST>, Box<RAST>, BinaryOperation),
    Unary(Box<RAST>, UnaryOperation),
    Atomic(u8),
}

pub fn parse(regex: &[Token]) -> Result<Box<RAST>, Error> {
    let mut regex: Vec<Token> = regex.iter().cloned().rev().collect();
    let rast = parse_regex(&mut regex)?;
    if !regex.is_empty() {
        return Err(Error::new("Regex stoped parsing before the end"));
    }
    Ok(Box::new(rast))
}

pub fn parse_regex(regex: &mut Vec<Token>) -> Result<RAST, Error> {
    parse_binary(regex)
}

fn parse_binary(regex: &mut Vec<Token>) -> Result<RAST, Error> {
    let unary = parse_unary(regex)?;
    if let Some(prime) = parse_binary_prime(regex)? {
        Ok(RAST::Binary(Box::new(unary), Box::new(prime.0), prime.1))
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
            Ok(Some((RAST::Binary(Box::new(unary), Box::new(prime.0), prime.1), token)))
        } else {
            Ok(Some((unary, token)))
        }
    } else {
        Ok(None)
    }
}

fn parse_unary(regex: &mut Vec<Token>) -> Result<RAST, Error> {
    let group = parse_group(regex)?;
    let opperation = parse_unary_prime(regex)?;
    Ok(if let Some(opperation) = opperation {
        RAST::Unary(Box::new(group), opperation)
    } else {
        group
    })
}

fn parse_unary_prime(regex: &mut Vec<Token>) -> Result<Option<UnaryOperation>, Error> {
    Ok(if let Some(t) = regex.pop() {
        match t {
            Token::KleenClosure     => Some(KleenClosure),
            Token::Question         => Some(Question),
            Token::Plus             => Some(Plus),
            Token::Times(min)       => Some(Times(min)),
            Token::MinMax(min, max) => Some(MinMax(min, max)),
            _ => {
                regex.push(t);
                None
            },
        }
    } else {
        None
    })
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

#[cfg(test)]
mod test {
    use super::*;
    use super::RAST::*;
    use crate::Error;
    use rand::Rng;

    #[test]
    fn basic() -> Result<(), Error> {
        let regex = "aa";
        let regex = crate::regex::get_rast(regex)?;
        assert_eq!(regex, Binary(
                Box::new(Atomic(97)), 
                Box::new(Atomic(97)), 
                Concat
        ));

        Ok(())
    }

    #[test]
    fn binary() -> Result<(), Error> {
        let regex = "aa|ab";
        let regex = crate::regex::get_rast(regex)?;
        let expected = 
            Binary(
                Box::new(Atomic(b'a')),
                Box::new(Binary(
                    Box::new(Atomic(b'a')),
                    Box::new(Binary(
                        Box::new(Atomic(b'a')),
                        Box::new(Atomic(b'b')),
                        Concat,
                    )),
                    Alternation,
                )),
                Concat,
            );
        assert_eq!(regex, expected);

        let regex = "(ab)|(cd)";
        let regex = crate::regex::get_rast(regex)?;
        let expected = 
            Binary(
                Box::new(Binary(
                    Box::new(Atomic(b'a')),
                    Box::new(Atomic(b'b')),
                    Concat,
                )),
                Box::new(Binary(
                    Box::new(Atomic(b'c')),
                    Box::new(Atomic(b'd')),
                    Concat,
                )),
                Alternation,
            )
        ;
        assert_eq!(regex, expected);

        Ok(())
    }

    #[test]
    fn unary() -> Result<(), Error> {
        let regex = "a+";
        let regex = crate::regex::get_rast(regex)?;
        let expected = Unary(Box::new(Atomic(b'a')), Plus);
        assert_eq!(regex, expected);

        let regex = "ab+";
        let regex = crate::regex::get_rast(regex)?;
        let expected = 
            Binary(
                Box::new(Atomic(b'a')),
                Box::new(Unary(
                    Box::new(Atomic(b'b')),
                    Plus
                )),
                Concat,
            )
        ;
        assert_eq!(regex, expected);

        let regex = "(ab)+";
        let regex = crate::regex::get_rast(regex)?;
        let expected = Unary(
            Box::new(Binary(
                Box::new(Atomic(b'a')),
                Box::new(Atomic(b'b')),
                Concat,
            )),
            Plus,
        );
        assert_eq!(regex, expected);

        Ok(())
    }

    #[test]
    #[allow(unused_must_use)]
    fn monkey() {
        let mut rng = rand::thread_rng();
        for _ in 0..10000 {
            let length = rng.gen_range(0, 16);
            let mut regex = String::new();
            for _ in 0..length {
                regex.push(rng.gen_range(32, 127) as u8 as char);
            }
            crate::regex::get_rast(&regex);
        }
    }
}

