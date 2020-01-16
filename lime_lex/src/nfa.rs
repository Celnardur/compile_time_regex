use std::{collections::HashMap, rc::Rc, cell::RefCell, collections::HashSet};
use crate::Error;

type Pointer = Rc<RefCell<Node>>;

#[derive(Debug)]
pub struct NFA {
    start: Pointer,
}

#[derive(Debug)]
struct Node {
    accepting: bool,
    links: HashMap<u8, Vec<Pointer>>,
}

impl Node {
    fn new(accepting: bool, link: Option<(u8, Node)>) -> Node {
        let mut node = Node {
            accepting,
            links: HashMap::new(), 
        };
        if let Some((c, n)) = link {
            node.links.insert(c, vec![Rc::new(RefCell::new(n))]);
        }
        node
    }

}

impl NFA {
    pub fn new(regex: &str) -> NFA {
        panic!("unimplmented");
    }

    fn from_tokens(regex: &[RegexToken]) -> Result<NFA, Error> {
        /*
        if regex.len() == 1 {
            let accepting = Node::new(true, None);
            let start = Node::new(false, Some((regex[0], accepting)));
            return Ok(NFA {
                start: Rc::new(RefCell::new(start)),
            })
        }
        */
        panic!("unimplmented");
    }
}    

#[derive(Copy, Clone)]
pub enum RegexToken {
    Character(u8),
    Alternation,
    KleenClosure,
    LParen,
    RParen,
}

/// This is basically a "hand-made" regex scanner
/// This takes complex regex, simplifies the rules '?', '+', and '[]', and tokenizes it.
/// s? is turned into (s|e) (where 'e' represents the empty set)
/// s+ becomes ss*
/// [a-z] becomes (a|b|c|...|z)
/// [^a-w] becomes (x|y|z)
/// So were left with a tokenized regex that only has alternation, concatenation, Kleen closure
/// ('*') and groupings. 
/// This only supports ascii. 
pub fn scan_regex(regex: &[u8]) -> Result<Vec<RegexToken>, Error> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < regex.len() {
        let c = regex[index] as char;
        match c {
            '\\' => {
                index += 1;
                tokens.push(RegexToken::Character(regex[index]));
            }, 
            '|' => tokens.push(RegexToken::Alternation),
            '*' => tokens.push(RegexToken::KleenClosure),
            '(' => tokens.push(RegexToken::LParen),
            ')' => tokens.push(RegexToken::RParen),
            '[' => {
                let (mut group, offset) = scan_set(&regex[(index + 1)..])?;
                tokens.append(&mut group);
                index += offset;
            },
            ']' => return Err(Error::new("Mismatched []")),
            '?' => {
                let insert_spot = tokens.len() - get_previous_group(&tokens[..])?.len();
                tokens.insert(insert_spot, RegexToken::LParen);
                tokens.push(RegexToken::Alternation);
                tokens.push(RegexToken::Character(0));
                tokens.push(RegexToken::RParen);
            },
            '+' => {
                let mut group = get_previous_group(&tokens[..])?.iter().cloned().collect();
                tokens.append(&mut group);
                tokens.push(RegexToken::KleenClosure);
            },
            _ => tokens.push(RegexToken::Character(regex[index])),
        }
        index += 1;
    }
    Ok(tokens)
}

/// looks backward from end to get the tokens from the previous group
fn get_previous_group(regex: &[RegexToken]) -> Result<&[RegexToken], Error> {
    match regex[regex.len() -1] {
        RegexToken::Character(c) => return Ok(&regex[(regex.len()-1)..regex.len()]),
        RegexToken::RParen => (),
        _ => return Err(Error::new("Cant use an operator on a non group")),
    }

    let mut depth = 1;
    let mut index = regex.len() - 2;
    while index >= 0 {
        let token = regex[index];
        match token {
            RegexToken::RParen => depth += 1,
            RegexToken::LParen => depth -= 1,
            _ => (),
        }
        if depth == 0 {
            return Ok(&regex[index..]);
        }
        index -= 1;
    }
    Err(Error::new("+ or ? used on group with no matching parens"))
}

/// This takes a slice that starts one after a [ and goes to the end of the regex
/// It returns The tokens that represent the alternation for that set 
/// and the number characters parsed from the original regex
fn scan_set(regex: &[u8]) -> Result<(Vec<RegexToken>, usize), Error> {
    // initialize variables
    let is_not = regex[0] as char == '^';
    let mut index = 0;
    if is_not { index += 1; }
    let mut tokens = Vec::new();

    // get a set of characters represented in the set
    let mut set = HashSet::new();
    while index < regex.len() && regex[index] as char != ']' {
        if regex[index] as char == '\\' { 
            set.insert(regex[index+1]);
            index +=2;
        } else if regex[index+1] as char == '-' {
            for c in regex[index]..regex[index+2] {
                set.insert(c);
            }
            index += 3;
        } else {
            set.insert(regex[index]);
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
        for i in 32..127 {
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
    tokens.push(RegexToken::LParen);
    for byte in set {
        tokens.push(RegexToken::Character(byte));
        tokens.push(RegexToken::Alternation);
    }
    tokens.pop();
    tokens.push(RegexToken::RParen);
    
    Ok((tokens, index+1))
}


