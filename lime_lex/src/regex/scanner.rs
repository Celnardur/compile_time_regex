use std::collections::{HashSet};
use crate::Error;


#[derive(Clone, Debug, PartialEq)]
pub enum RegexToken {
    Character(u8),
    Digit(u8),
    Alternation,
    KleenClosure,
    Question,
    Plus,
    Wildcard,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LCurly,
    RCurly,
    Comma,
}

use RegexToken::*;


pub fn scan(regex: &str) -> Result<Vec<FirstRegexToken>, Error> {
    if !regex.is_ascii() {
        return Err(Error::new("This Regex Engine only supports ASCII"));
    }
    let mut regex: Vec<u8> = regex.as_bytes().iter().cloned().rev().collect();
    if regex.len() == 0 {
        return Err(Error::new("Cannot have an empty regex"));
    }
    let mut tokens = Vec::new();
    while let Some(t) = scan_token(&mut regex)? {
        tokens.push(t);
    }
    Ok(tokens)
}

fn scan_token(regex: &mut Vec<u8>) -> Result<Option<FirstRegexToken>, Error> {
    let c = regex.pop();
    if c.is_none() {
        return Ok(None);
    }
    let c = c.unwrap();
    match c {
        b'\\' => {
            if let Some(c) = regex.pop() {
                Ok(Some(Character(get_escape_char(c))))
            } else {
                Err(Error::new("Cannot have \\ on end of regex"))
            }
        },
        b'|' => Ok(Some(Alternation)),
        b'*' => Ok(Some(KleenClosure)),
        b'?' => Ok(Some(Question)),
        b'+' => Ok(Some(Plus)),
        b'(' => Ok(Some(LParen)),
        b')' => Ok(Some(RParen)),
        b'{' => Ok(Some(LCurly)),
        b'}' => Ok(Some(RCurly)),
        b'[' => Ok(Some(LBracket)),
        b']' => Ok(Some(RBracket)),
        b'.' => Ok(Some(Wildcard)),
        b',' => Ok(Some(Comma)),
        b'0' => Ok(Some(Digit(0))),

        _ => Ok(Some(Character(c))),
    }
}
