
mod scan {
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
            b'{' => {
                let min = get_num(regex)?;
                if let Some(c) = regex.pop() {
                    match c {
                        b'}' => Ok(Some(Times(min))),
                        b',' => {
                            let max = get_num(regex)?;
                            if let Some(c) = regex.pop() {
                                if c == b'}' {
                                    Ok(Some(MinMax(min, max)))
                                } else {
                                    Err(Error::new("Mismatched {}"))
                                }
                            } else {
                                Err(Error::new("Regex ends without closing {"))
                            }
                        },
                        _ => Err(Error::new("Illegal character in brackets")),
                    }
                } else {
                    Err(Error::new("Regex ends without closing {"))
                }
            }, 
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

    fn get_num(regex: &mut Vec<u8>) -> Result<u8, Error> {
        let mut digits: Vec<u16> = Vec::new();
        while let Some(c) = regex.pop() {
            if c < 0x30 || c > 0x39 {
                regex.push(c);
                break;
            }
            digits.push((c & 0x0f) as u16);
        }
        if digits.len() > 3 {
            return Err(Error::new("{} opperation doesn't support numbers greater than 255"))
        }

        let mut acc: u16 = 0;
        let mut mult = 1;
        for digit in digits.iter().rev() {
            acc += (digit * mult) as u16;
            mult *= 10;
        }
        if acc > 255 {
            return Err(Error::new("{} opperation doesn't support numbers greater than 255"));
        }
        Ok(acc as u8)
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
        use super::FirstRegexToken::*;

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
        fn monkey() {
            let mut rng = rand::thread_rng();
            for _ in 0..10000 {
                let length = rng.gen_range(0,16);
                let mut regex = String::new();
                for _ in 0..length {
                    regex.push(rng.gen_range(32, 127) as u8 as char);
                }
                scan(&regex);
            }
        }

    }
}

mod simpilfy {
    use super::scan::FirstRegexToken;
    use std::{collections::HashSet};
    use crate::Error;
    use RegexToken::*;

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum RegexToken {
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
    pub fn simpilfy(regex: &[FirstRegexToken]) -> Result<Vec<RegexToken>, Error> {
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

    fn first_is_normal(tokens: &mut Vec<RegexToken>, second: RegexToken, index: usize) {
        match second {
            Character(_) => tokens.insert(index, Concat),
            LParen => tokens.insert(index, Concat),
            _ => (),
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use super::RegexToken::*;
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
        fn monkey() {
            let mut rng = rand::thread_rng();
            for _ in 0..10000 {
                let length = rng.gen_range(0,16);
                let mut regex = String::new();
                for _ in 0..length {
                    regex.push(rng.gen_range(32, 127) as u8 as char);
                }
                if let Ok(regex) = super::super::scan::scan(&regex) {
                    simpilfy(&regex[..]);
                }

            }
        }
    }
}

mod parse {
    use super::simpilfy::RegexToken;
    use crate::Error;
    use std::{rc::Rc, cell::RefCell};
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
    
    pub fn parse(regex: &[RegexToken]) -> Result<RAST, Error> {
        let mut regex: Vec<RegexToken> = regex.iter().cloned().rev().collect();
        parse_binary(&mut regex)
    }
    
    pub fn parse_regex(regex: &mut Vec<RegexToken>) -> Result<RAST, Error> {
        parse_binary(regex)
    }

    fn parse_binary(regex: &mut Vec<RegexToken>) -> Result<RAST, Error> {
        let unary = parse_unary(regex)?;
        if let Some(prime) = parse_binary_prime(regex)? {
            Ok(RAST::Binary(Rc::new(unary), Rc::new(prime.0), prime.1))
        } else {
            Ok(unary)
        }
    }

    fn parse_binary_prime(regex: &mut Vec<RegexToken>) -> Result<Option<(RAST, BinaryOperation)>, Error> {
        if let Some(t) = regex.pop() {
            let token = match t {
                RegexToken::Concat => Concat,
                RegexToken::Alternation => Alternation,
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
    
    fn parse_unary(regex: &mut Vec<RegexToken>) -> Result<RAST, Error> {
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

    fn parse_unary_prime(regex: &mut Vec<RegexToken>) -> Result<Vec<UnaryOperation>, Error> {
        if let Some(t) = regex.pop() {
            let token = match t {
                RegexToken::KleenClosure => KleenClosure,
                RegexToken::Question => Question,
                RegexToken::Plus => Plus,
                RegexToken::Times(min) => Times(min),
                RegexToken::MinMax(min, max) => MinMax(min, max),
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
        
    fn parse_group(regex: &mut Vec<RegexToken>) -> Result<RAST, Error> {
        if let Some(t) = regex.pop() {
            match t {
                RegexToken::Character(c) => Ok(RAST::Atomic(c)),
                RegexToken::LParen => {
                    let group = parse_regex(regex)?;
                    if let Some(t) = regex.pop() {
                        match t {
                            RegexToken::RParen => Ok(group),
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
}

