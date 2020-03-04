use std::{collections::HashSet};
use crate::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum FirstRegexToken {
    Character(u8),
    MinMax(u8, u8),
    Times(u8),
    Set(HashSet<u8>),
    InverseSet(HashSet<u8>),
    Alternation,
    KleenClosure,
    Question,
    Plus,
    Wildcard,
    LParen,
    RParen,
}

use FirstRegexToken::*;

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
        b'{' => scan_times(regex), 
        b'[' => {
            if let Some(c) = regex.pop() {
                if c == b'^' {
                    Ok(Some(InverseSet(get_set(regex)?)))
                } else {
                    regex.push(c);
                    Ok(Some(Set(get_set(regex)?)))
                }
            } else {
                Err(Error::new("Mismatched []"))
            }
        },
        b'.' => Ok(Some(Wildcard)),
        _ => Ok(Some(Character(c))),
    }
}

fn get_escape_char(letter: u8) -> u8 {
    match letter {
        b'0' => 0,
        b'r' => 13,
        b'n' => 10,
        b't' => 9,
        _ => letter,
    }
}

fn scan_times(regex: &mut Vec<u8>) -> Result<Option<FirstRegexToken>, Error> {
    // get first number in 
    let min = get_num(regex)?;

    // check for closing } (times token) or , (min, max token)
    let c = regex.pop();
    if c == None {
        return Err(Error::new("Regex ends without closing {"));
    }
    match c.unwrap() {
        b'}' => return Ok(Some(Times(min))),
        b',' => (),
        _ => return Err(Error::new("Illegal character in brackets")),
    }

    // get max for min max
    let max = get_num(regex)?;

    // make sure it has closing }
    if let Some(c) = regex.pop() {
        if c == b'}' {
            Ok(Some(MinMax(min, max)))
        } else {
            Err(Error::new("Mismatched {}"))
        }
    } else {
        Err(Error::new("Regex ends without closing {"))
    }
}

fn get_num(regex: &mut Vec<u8>) -> Result<u8, Error> {
    if regex.is_empty() {
        return Err(Error::new("Mismatched {"));
    }

    let mut number: u64 = 0;
    while let Some(c) = regex.pop() {
        if c < 0x30 || c > 0x39 {
            regex.push(c);
            break;
        }
        number = (number * 10) + ((c & 0x0f) as u64);
    }

    if number > 255 {
        return Err(Error::new("Numbers in {} must be less than 256"));
    }
    Ok(number as u8)
}

fn get_set(regex: &mut Vec<u8>) -> Result<HashSet<u8>, Error> {
    let mut set = HashSet::new();
    while let Some(c) = regex.pop() {
        match c {
            b'\\' => {
                if let Some(c) = regex.pop() {
                    regex.push(get_escape_char(c));
                } else {
                    return Err(Error::new("Cannot have \\ on end of regex"));
                }
            },
            b']' => break,
            _ => {
                let first = c;
                if let Some(c) = regex.pop() {
                    match c {
                        b']' => {
                            set.insert(first);
                            break;
                        },
                        b'-' => {
                            if let Some(c) = regex.pop() {
                                for i in first..(c+1) {
                                    set.insert(i);
                                }
                            } else {
                                return Err(Error::new("Mismatched []"));
                            }
                        },
                        _ => {
                            set.insert(first);
                            regex.push(c);
                        },
                    }
                } else {
                    return Err(Error::new("Mismatched []"))
                }
            }
        }
    }
    Ok(set)
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;
    use crate::Error;

    #[test] 
    fn basic() -> Result<(), Error> {
        let regex = r"\||*?+().a";
        let tokens = scan(regex)?;
        assert_eq!(tokens, [Character(b'|'), Alternation, KleenClosure, Question, Plus, LParen, RParen, Wildcard, Character(b'a')]);
        Ok(())
    }

    #[test]
    fn sets() -> Result<(), Error> {
        let regex = r"[a-c]";
        let tokens = scan(regex)?;
        assert_eq!(tokens.len(), 1);
        let token = tokens[0].clone();
        match token {
            Set(s) => {
                assert_eq!(s.len(), 3);
                assert!(s.contains(&b'a'));
                assert!(s.contains(&b'b'));
                assert!(s.contains(&b'c'));
            },
            _ => panic!("Unexpected token")
        }

        let regex = r"[^a-c]";
        let tokens = scan(regex)?;
        assert_eq!(tokens.len(), 1);
        let token = tokens[0].clone();
        match token {
            InverseSet(s) => {
                assert_eq!(s.len(), 3);
                assert!(s.contains(&b'a'));
                assert!(s.contains(&b'b'));
                assert!(s.contains(&b'c'));
            },
            _ => panic!("Unexpected token")
        }

        Ok(())
    }

    #[test]
    fn brakcets() -> Result<(), Error> {
        let regex = r"a{3}";
        let tokens = scan(regex)?;
        assert_eq!(tokens, [Character(b'a'), Times(3)]);

        let regex = r"a{3,5}";
        let tokens = scan(regex)?;
        assert_eq!(tokens, [Character(b'a'), MinMax(3, 5)]);
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
            scan(&regex); // result needs to be unused
        }
    }
}
