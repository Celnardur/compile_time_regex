use super::scan::FirstRegexToken;
use std::{collections::HashSet};
use crate::Error;
use Token::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Token {
    Character(u8),
    MinMax(u8, u8),
    Times(u8),
    Concat,
    Alternation,
    KleenClosure,
    Question,
    Plus,
    LParen,
    RParen,
}

/// Simpilifies Set, InversSet, and Wildcard and adds Concat operator
pub fn simpilfy(regex: &[FirstRegexToken]) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
    let mut regex: Vec<FirstRegexToken> = regex.iter().cloned().rev().collect();

    // Simpilfy pass
    while let Some(t) = regex.pop() {
        match t {
            FirstRegexToken::Set(hs) => {
                if hs.is_empty() {
                    return Err(Error::new("Cannot have an empty set []"))
                }
                tokens.push(LParen);
                for byte in hs {
                    tokens.push(Character(byte));
                    tokens.push(Alternation);
                }
                tokens.pop();
                tokens.push(RParen);
            },
            FirstRegexToken::InverseSet(set) => {
                let mut new_set = HashSet::new();
                // sorry ascii only
                for i in 0..127 {
                    if !set.contains(&i) {
                        new_set.insert(i);
                    }
                }
                let hs = new_set;
                if hs.is_empty() {
                    return Err(Error::new("Cannot have an empty set []"))
                }
                tokens.push(LParen);
                for byte in hs {
                    tokens.push(Character(byte));
                    tokens.push(Alternation);
                }
                tokens.pop();
                tokens.push(RParen);
            },
            FirstRegexToken::Wildcard => {
                tokens.push(LParen);
                for byte in 0..127 {
                    tokens.push(Character(byte));
                    tokens.push(Alternation);
                }
                tokens.pop();
                tokens.push(RParen);
            }
            FirstRegexToken::Character(c) => tokens.push(Character(c)),
            FirstRegexToken::MinMax(min, max) => tokens.push(MinMax(min, max)),
            FirstRegexToken::Times(min) => tokens.push(Times(min)),
            FirstRegexToken::Alternation => tokens.push(Alternation),
            FirstRegexToken::KleenClosure => tokens.push(KleenClosure),
            FirstRegexToken::Question => tokens.push(Question),
            FirstRegexToken::Plus => tokens.push(Plus),
            FirstRegexToken::LParen => tokens.push(LParen),
            FirstRegexToken::RParen => tokens.push(RParen),
        }
    }

    // add concatination pass
    let mut index = 0;
    while index + 1 < tokens.len() {
        let first = tokens[index];
        let second = tokens[index + 1];

        match first {
            Character(_) => first_is_normal(&mut tokens, second, index+1),
            MinMax(_, _) => first_is_normal(&mut tokens, second, index+1),
            Times(_) => first_is_normal(&mut tokens, second, index+1),
            KleenClosure => first_is_normal(&mut tokens, second, index+1),
            Question => first_is_normal(&mut tokens, second, index+1),
            Plus => first_is_normal(&mut tokens, second, index+1),
            RParen => first_is_normal(&mut tokens, second, index+1),
            _ => (),
        }
        index += 1;
    }
    
    Ok(tokens)
}

fn first_is_normal(tokens: &mut Vec<Token>, second: Token, index: usize) {
    match second {
        Character(_) => tokens.insert(index, Concat),
        LParen => tokens.insert(index, Concat),
        _ => (),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;

    #[test]
    fn basic() -> Result<(), Error> {
        let regex = "aa";
        let regex = super::super::scan::scan(regex)?;
        let tokens = simpilfy(&regex[..])?;
        assert_eq!(tokens, [Character(b'a'), Concat, Character(b'a')]);
        Ok(()) 
    }

    #[test] 
    fn swaping() -> Result<(), Error> {
        let regex = "[a-c]";
        let regex = super::super::scan::scan(regex)?;
        let tokens = simpilfy(&regex[..])?;
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], LParen);
        assert_eq!(tokens[6], RParen);
        assert_eq!(tokens[2], Alternation);
        assert_eq!(tokens[4], Alternation);
        assert!(tokens.contains(&Character(b'a')));
        assert!(tokens.contains(&Character(b'b')));
        assert!(tokens.contains(&Character(b'c')));

        let regex = "[^a-c]";
        let regex = super::super::scan::scan(regex)?;
        let tokens = simpilfy(&regex[..])?;
        assert!(tokens.len() > 100);
        assert!(!tokens.contains(&Character(b'a')));
        assert!(!tokens.contains(&Character(b'b')));
        assert!(!tokens.contains(&Character(b'c')));

        Ok(())
    }

    #[test]
    fn concat() -> Result<(), Error> {
        let regex = "a*a";
        let regex = super::super::scan::scan(regex)?;
        let tokens = simpilfy(&regex[..])?;
        assert_eq!(tokens, [Character(b'a'), KleenClosure, Concat, Character(b'a')]);

        let regex = "a*(a)";
        let regex = super::super::scan::scan(regex)?;
        let tokens = simpilfy(&regex[..])?;
        assert_eq!(tokens, [Character(b'a'), KleenClosure, Concat, LParen, Character(b'a'), RParen]);
        Ok(()) 
    }

    #[test]
    #[allow(unused_must_use)]
    fn monkey() {
        let mut rng = rand::thread_rng();
        for _ in 0..10000 {
            let length = rng.gen_range(0,16);
            let mut regex = String::new();
            for _ in 0..length {
                regex.push(rng.gen_range(32, 127) as u8 as char);
            }
            if let Ok(regex) = super::super::scan::scan(&regex) {
                simpilfy(&regex[..]); // result needs to be unused
            }
        }
    }
}
