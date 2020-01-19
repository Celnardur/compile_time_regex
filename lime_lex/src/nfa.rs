/*
use std::{collections::HashMap, rc::Rc, cell::RefCell, collections::HashSet};
use crate::Error;
use crate::regex;
use crate::regex::RegexToken;
use crate::regex::RegexToken::*;

type Pointer = Rc<RefCell<Node>>;

fn np(node: Node) -> Pointer {
    Rc::new(RefCell::new(node))
}

fn npn(link: Option<(u8, &Pointer)>) -> Pointer {
    Rc::new(RefCell::new(Node::new(link)))
}

fn cp(pointer: &Pointer) -> Pointer {
    Rc::clone(&pointer)
}

const ASCII_MAX: u8 = 126;

#[derive(Debug, Clone)]
pub struct NFA {
    start: Pointer,
    end: Pointer,
}

#[derive(Debug, Clone)]
struct Node {
    links: Vec<Destinations>,
    epsilon: Destinations,
}

#[derive(Debug, Clone)]
enum Destinations {
    Nowhere,
    Places(Vec<Pointer>),
}

use Destinations::*;

impl Node {
    fn new(link: Option<(u8, &Pointer)>) -> Node {
        let mut node = Node {
            links: Vec::new(), 
            epsilon: Nowhere,
        };
        node.links.resize(ASCII_MAX as usize, Nowhere);
        if let Some((c, n)) = link {
            node.links[c as usize] = Places(vec![cp(&n)]);
        }
        node
    }
}

impl NFA {
    pub fn new(regex: &str) -> Result<NFA, Error> {
        let tokens = regex::scan_regex(regex)?;

        panic!("unimplmented");
    }

    fn from_tokens(regex: &[RegexToken]) -> Result<NFA, Error> {
        if regex.is_empty() {
            panic!("This Should never happen")
        }

        if regex.len() == 1 {
            return match regex[0] {
                Character(c) => {
                    let accepting = npn(None);
                    let start = npn(Some((c, &accepting)));
                    Ok(NFA {
                        start: cp(&start),
                        end: cp(&accepting),
                    })
                },
                Epsilon => {
                    let accepting = npn(None);
                    let start = npn(None);
                    start.borrow_mut().epsilon = Places(vec![cp(&accepting)]);
                    Ok(NFA {
                        start: cp(&start),
                        end: cp(&accepting),
                    })
                },
                _ => Err(Error::new("Regex Is Illegal")),
            }
        }

        let (first, remaining) = match regex[0] {
            Character(_) => (NFA::from_tokens(&regex[0..1])?, &regex[1..]),
            Epsilon => (NFA::from_tokens(&regex[0..1])?, &regex[1..]),
            _ => return Err(Error::new("Illegal start token")),
        };
        panic!("unimplmented");
    }
}    
*/