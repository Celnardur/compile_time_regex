
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
        fn scan_test_basic() -> Result<(), Error> {
            let regex = r"\||*?+().a";
            let tokens = scan(regex)?;
            assert_eq!(tokens, [Character(b'|'), Alternation, KleenClosure, Question, Plus, LParen, RParen, Wildcard, Character(b'a')]);
            Ok(())
        }

        #[test]
        fn scan_test_sets() -> Result<(), Error> {
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
        fn scan_test_brakcets() -> Result<(), Error> {
            let regex = r"a{3}";
            let tokens = scan(regex)?;
            assert_eq!(tokens, [Character(b'a'), Times(3)]);

            let regex = r"a{3,5}";
            let tokens = scan(regex)?;
            assert_eq!(tokens, [Character(b'a'), MinMax(3, 5)]);
            Ok(())
        }

        #[test]
        fn scan_test_monkey() {
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






/*
/// This is basically a "hand-made" regex scanner
/// This takes complex regex, simplifies the rules '?', '+', and '[]', and tokenizes it.
/// s? is turned into (s|e) (where 'e' represents the empty set)
/// s+ becomes ss*
/// [a-z] becomes (a|b|c|...|z)
/// [^a-w] becomes (x|y|z) (where the set is lower case letters)
/// So were left with a tokenized regex that only has alternation, concatenation, Kleen closure
/// ('*') and groupings. 
/// This only supports ascii. 
pub fn scan_regex(regex: &str) -> Result<Vec<FirstRegexToken>, Error> {
    if !regex.is_ascii() {
        return Err(Error::new("This Regex Engine only supports ASCII"));
    }

    let regex = regex.as_bytes();
    if regex.len() == 0 {
        return Err(Error::new("Cannot have an empty regex"));
    }
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < regex.len() {
        let c = regex[index] as char;
        match c {
            '\\' => {
                index += 1;
                if index >= regex.len() {
                    return Err(Error::new("'\\' can't be last character in regex"));
                }
                tokens.push(FirstRegexToken::Character(get_escape_char(regex[index])));
            }, 
            '|' => tokens.push(Alternation),
            '*' => tokens.push(KleenClosure),
            '(' => tokens.push(LParen),
            ')' => tokens.push(RParen),
            '[' => {
                let (mut group, offset) = scan_set(&regex[(index + 1)..])?;
                tokens.append(&mut group);
                index += offset;
            },
            ']' => return Err(Error::new("Mismatched []")),
            '?' => {
                let insert_spot = tokens.len() - get_previous_group(&tokens[..])?.len();
                tokens.insert(insert_spot, FirstRegexToken::LParen);
                tokens.push(Alternation);
                tokens.push(Epsilon);
                tokens.push(RParen);
            },
            '+' => {
                let mut group = get_previous_group(&tokens[..])?.iter().cloned().collect();
                tokens.append(&mut group);
                tokens.push(KleenClosure);
            },
            '.' => {
                let mut dot = scan_regex(r"[\0-~]")?;
                tokens.append(&mut dot);
            }
            '{' => {
                let mut at = index + 1;
                while regex[index] != '}' as u8 {
                    if index + 1 >= regex.len() {
                        return Err(Error::new("Mismatched {}"));
                    }
                    index += 1;
                }
                index += 1;
                let (start, len) = get_next_num(&regex[at..])?;
                at += len as usize;
                let group: Vec<FirstRegexToken> = get_previous_group(&tokens[..])?.iter().cloned().collect();
                match regex[at] as char {
                    '}' => {
                        if start < 2 {
                            return Err(Error::new("Variable in bracket makes no sense"));
                        }
                        for _ in 1..start {
                            tokens.append(&mut group.clone());
                        }
                    },
                    ',' => {
                        at += 1;
                        let (end, len) = get_next_num(&regex[at..])?;
                        at += len as usize;
                        if regex[at] != '}' as u8 {
                            return Err(Error::new("Illegal patern in {}"));
                        }
                        tokens.push(MinMax(start, end));
                    },
                    _ => return Err(Error::new("{} Pattern is not upheld"))
                }

            }
            _ => tokens.push(Character(regex[index])),
        }
        index += 1;
    }
    Ok(tokens)
}

fn get_next_num(regex: &[u8]) -> Result<(u8, u8), Error> {
   let mut digits = Vec::new();
    if regex.is_empty() || regex[0] < 0x30 || regex[0] > 0x39 {
        return Err(Error::new("Bracket contains non number in illeagal spot"));
    }
    let mut index = 0;
    while regex[index] >= 0x30 && regex[index] <= 0x39 {
        digits.push(regex[index] & 0x0f);
        index += 1;
    }
    if digits.len() > 3 {
        return Err(Error::new("Cannoth have {} be greater than 255 sorry"));
    }
    let mut acc: u16 = 0;
    let mut mult = 1;
    for digit in digits.iter().rev() {
        acc += (digit * mult) as u16;
        mult *= 10;
    }
    if acc > 255 {
        return Err(Error::new("Cannoth have {} be greater than 255 sorry"));
    }
    Ok((acc as u8, digits.len() as u8))
}

/// looks backward from end to get the tokens from the previous group
fn get_previous_group(regex: &[FirstRegexToken]) -> Result<&[FirstRegexToken], Error> {
    if regex.is_empty() {
        return Err(Error::new("Can't use + or ? operator at the begining of a regex"));
    }

    match regex[regex.len() -1] {
        Character(c) => return Ok(&regex[(regex.len()-1)..regex.len()]),
        RParen => (),
        _ => return Err(Error::new("Cant use an operator on a non group")),
    }

    let mut depth = 1;
    let mut index = regex.len() - 1;
    while index > 0 {
        index -= 1;

        let token = regex[index].clone();
        match token {
            RParen => depth += 1,
            LParen => depth -= 1,
            _ => (),
        }
        if depth == 0 {
            return Ok(&regex[index..]);
        }
    }
    Err(Error::new("+ or ? used on group with no matching parens"))
}

/// This takes a slice that starts one after a [ and goes to the end of the regex
/// It returns The tokens that represent the alternation for that set 
/// and the number characters parsed from the original regex
fn scan_set(regex: &[u8]) -> Result<(Vec<FirstRegexToken>, usize), Error> {
    if regex.len() == 0 {
        return Err(Error::new("Mismatched []"));
    }

    // initialize variables
    let is_not = regex[0] as char == '^';
    let mut index = 0;
    if is_not { index += 1; }
    let mut tokens = Vec::new();

    // get a set of characters represented in the set
    let mut set = HashSet::new();
    while index < regex.len() && regex[index] as char != ']' {
        let mut from = regex[index];
        if index + 1 < regex.len() && regex[index] as char == '\\' { 
            from = get_escape_char(regex[index+1]);
            index +=1;
        } 
        
        if index + 2 < regex.len() && regex[index+1] as char == '-' {
            let mut to = regex[index+2];
            if index + 3 < regex.len() && to as char == '\\' { 
                to = get_escape_char(regex[index+3]);
                index +=1;
            } 

            for c in from..(to+1) {
                set.insert(c);
            }
            index += 3;
        } else {
            set.insert(from);
            index += 1;
        }
    }
    if index == regex.len() {
        return Err(Error::new("Mismatched \"[]\""));
    }

    // invert set if [^ 
    if is_not {
        let mut new_set = HashSet::new();
        // sorry ascii only
        for i in 0..127 {
            if !set.contains(&i) {
                new_set.insert(i);
            }
        }
        set = new_set;
    }

    // cannot have an empty bracketed set 
    if set.len() == 0 {
        let internals = std::str::from_utf8(&regex[0..index]).unwrap();
        return Err(Error::new(&format!("[{}] in describes an illegal empty set", internals)));
    }

    // create alternation of set
    tokens.push(LParen);
    for byte in set {
        tokens.push(Character(byte));
        tokens.push(Alternation);
    }
    tokens.pop();
    tokens.push(RParen);
    
    Ok((tokens, index+1))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Error;
    use super::FirstRegexToken::*;
    use rand::Rng;
    
    #[test]
    fn scan_test_basic() -> Result<(),Error> {
        let regex = "a";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens[0], Character('a' as u8));

        let regex = "(a|b)a*";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens, [LParen, Character('a' as u8), Alternation, Character('b' as u8), 
        RParen, Character('a' as u8), KleenClosure]);

        // tests \ as escape
        let regex = r"\*(\))";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens, [Character('*' as u8), LParen, Character(')' as u8), RParen]);

        Ok(())
    }

    #[test]
    fn scan_test_set() -> Result<(), Error> {
        // basic test
        let regex = "[a-c]";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], LParen);
        assert_eq!(tokens[6], RParen);
        assert_eq!(tokens[2], Alternation);
        assert_eq!(tokens[4], Alternation);
        assert!(tokens.contains(&Character('a' as u8)));
        assert!(tokens.contains(&Character('b' as u8)));
        assert!(tokens.contains(&Character('c' as u8)));

        // test concatinating sets
        let regex = "[_a-c]";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens.len(), 9);
        assert!(tokens.contains(&Character('_' as u8)));
        assert!(tokens.contains(&Character('a' as u8)));
        assert!(tokens.contains(&Character('b' as u8)));
        assert!(tokens.contains(&Character('c' as u8)));

        // test escape sequence
        let regex = r"[\^\-a-c]";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens.len(), 11);
        assert!(tokens.contains(&Character('-' as u8)));
        assert!(tokens.contains(&Character('^' as u8)));

        Ok(())
    }

    #[test]
    fn scan_test_inverse_set() -> Result<(), Error> {
        // test inverse set
        let regex = r"[^\0-`d-~]";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], LParen);
        assert_eq!(tokens[6], RParen);
        assert_eq!(tokens[2], Alternation);
        assert_eq!(tokens[4], Alternation);
        assert!(tokens.contains(&Character('a' as u8)));
        assert!(tokens.contains(&Character('b' as u8)));
        assert!(tokens.contains(&Character('c' as u8)));

        Ok(())
    }

    #[test]
    fn scan_test_option() -> Result<(), Error> {
        // basic ?
        let regex = r"a?";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens, [LParen, Character('a' as u8), Alternation, Epsilon, RParen]);

        // ? on group
        let regex = r"foo(ab)?asdf";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens, [Character('f' as u8), Character('o' as u8), Character('o' as u8), LParen, 
                   LParen, Character('a' as u8), Character('b' as u8), RParen,
                   Alternation, Epsilon, RParen, 
                   Character('a' as u8), Character('s' as u8), Character('d' as u8), Character('f' as u8),
        ]);

        // nested parens
        let regex = r"f((a|b)(c|d))?a";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens.len(), 18);
        assert_eq!(tokens[1], LParen);
        assert_eq!(tokens[2], LParen);
        assert_eq!(tokens[3], LParen);
        assert_eq!(tokens[4], Character('a' as u8));

        Ok(())
    }

    #[test]
    fn scan_test_plus() -> Result<(), Error> {
        // basic +
        let regex = r"a+";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens, [Character('a' as u8), Character('a' as u8), KleenClosure]);

        // + on group
        let regex = r"foo(ab)+asdf";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens, [Character('f' as u8), Character('o' as u8), Character('o' as u8),
                   LParen, Character('a' as u8), Character('b' as u8), RParen,
                   LParen, Character('a' as u8), Character('b' as u8), RParen,
                   KleenClosure,
                   Character('a' as u8), Character('s' as u8), Character('d' as u8), Character('f' as u8),
        ]);

        // nested parens
        let regex = r"f((a|b)(c|d))+a";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens.len(), 27);

        Ok(())
    }

    #[test]
    fn scan_test_bracket() -> Result<(), Error> {
        let regex = r"a{3}";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens, [Character('a' as u8), Character('a' as u8), Character('a' as u8)]);

        let regex = r"a{3,7}";
        let tokens = scan_regex(regex)?;
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens, [Character('a' as u8), MinMax(3, 7)]);

        Ok(())
    }

    #[test]
    fn scan_test_errors() {
        let regex = r"]";
        match scan_regex(regex) {
            Err(e) => (),
            Ok(_) => panic!("Regex {} should have produced a scan error", regex), 
        }

        let regex = r"]+";
        match scan_regex(regex) {
            Err(e) => (),
            Ok(_) => panic!("Regex {} should have produced a scan error", regex), 
        }

        let regex = r"())?";
        match scan_regex(regex) {
            Err(e) => (),
            Ok(_) => panic!("Regex {} should have produced a scan error", regex), 
        }

        let regex = r"[";
        match scan_regex(regex) {
            Err(e) => (),
            Ok(_) => panic!("Regex {} should have produced a scan error", regex), 
        }

        let regex = r"a[]b";
        match scan_regex(regex) {
            Err(e) => (),
            Ok(_) => panic!("Regex {} should have produced a scan error", regex), 
        }
    }

    #[test]
    fn scan_test_monkey() {
        let mut rng = rand::thread_rng();
        for _ in 0..10000 {
            let length = rng.gen_range(0,16);
            let mut regex = String::new();
            for _ in 0..length {
                regex.push(rng.gen_range(32, 127) as u8 as char);
            }
            scan_regex(&regex);
        }
    }
}
*/