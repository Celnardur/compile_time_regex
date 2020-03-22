use crate::regex::parse::BinaryOperation;
use crate::regex::parse::UnaryOperation;
use crate::regex::parse::RAST;
use BinaryOperation::*;
use Transition::*;
use UnaryOperation::*;
use RAST::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Transition {
    Epsilon(Vec<usize>),
    Character(u8, usize),
}

// first element is the start node
// last element is the finish node
pub type NFA = Vec<Transition>;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Range {
    start: usize,
    end: usize,
}

impl Transition {
    fn add_epsilon(&mut self, to: usize) {
        match self {
            Epsilon(transitions) => transitions.push(to),
            _ => panic!("Programmer Error: Should never add epsilon transitions to non-epsilon"),
        }
    }
}

fn new_epsilon(nfa: &mut NFA, transitions: Vec<usize>) -> usize {
    nfa.push(Epsilon(transitions));
    nfa.len() - 1
}

fn add_nfa(nfa: &mut NFA, mut to_insert: NFA) -> Range {
    for transition in &mut to_insert {
        match transition {
            Epsilon(to) => {
                for pos in to {
                    *pos += nfa.len();
                }
            }
            Character(_, to) => *to += nfa.len(),
        }
    }
    let start = nfa.len();
    nfa.append(&mut to_insert);
    Range {
        start,
        end: nfa.len() - 1,
    }
}

pub fn rast_to_nfa(rast: &RAST) -> NFA {
    match rast {
        Atomic(atomic) => vec![Character(*atomic, 1), Epsilon(Vec::new())],
        Binary(left, right, op) => construct_binary_op(left, right, *op),
        Unary(rast, op) => construct_unary_op(rast, *op),
    }
}

fn construct_binary_op(left: &RAST, right: &RAST, op: BinaryOperation) -> NFA {
    let mut nfa = Vec::new();

    match op {
        Concat => {
            let left = add_nfa(&mut nfa, rast_to_nfa(left));
            let right = add_nfa(&mut nfa, rast_to_nfa(right));
            nfa[left.end].add_epsilon(right.start);
        }
        Alternation => {
            let start = new_epsilon(&mut nfa, Vec::new());
            let left = add_nfa(&mut nfa, rast_to_nfa(left));
            let right = add_nfa(&mut nfa, rast_to_nfa(right));
            let end = new_epsilon(&mut nfa, Vec::new());
            nfa[start].add_epsilon(left.start);
            nfa[start].add_epsilon(right.start);
            nfa[left.end].add_epsilon(end);
            nfa[right.end].add_epsilon(end);
        }
    }
    nfa
}

fn construct_unary_op(rast: &RAST, op: UnaryOperation) -> NFA {
    let mut nfa = Vec::new();
    let middle = rast_to_nfa(rast);

    match op {
        KleenClosure => {
            let start = new_epsilon(&mut nfa, Vec::new());
            let middle = add_nfa(&mut nfa, middle);
            let end = new_epsilon(&mut nfa, vec![start]);
            nfa[start].add_epsilon(middle.start);
            nfa[start].add_epsilon(end);
            nfa[middle.end].add_epsilon(end);
        }
        Question => {
            let start = new_epsilon(&mut nfa, Vec::new());
            let middle = add_nfa(&mut nfa, middle);
            let end = new_epsilon(&mut nfa, Vec::new());
            nfa[start].add_epsilon(middle.start);
            nfa[start].add_epsilon(end);
            nfa[middle.end].add_epsilon(end);
        }
        Plus => {
            let first = add_nfa(&mut nfa, middle.clone());
            let start = new_epsilon(&mut nfa, Vec::new());
            nfa[first.end].add_epsilon(start);
            let middle = add_nfa(&mut nfa, middle);
            let end = new_epsilon(&mut nfa, vec![start]);
            nfa[start].add_epsilon(middle.start);
            nfa[start].add_epsilon(end);
            nfa[middle.end].add_epsilon(end);
        }
        Times(times) => {
            let mut at = add_nfa(&mut nfa, middle.clone());
            // start from one because at is already the first one added
            for _ in 1..times {
                let next = add_nfa(&mut nfa, middle.clone());
                nfa[at.end].add_epsilon(next.start);
                at = next;
            }
        }
        MinMax(min, max) => {
            let mut at = add_nfa(&mut nfa, middle.clone());
            // start from one because at is already the first one added
            for _ in 1..min {
                let next = add_nfa(&mut nfa, middle.clone());
                nfa[at.end].add_epsilon(next.start);
                at = next;
            }
            let mut hook_to_end = Vec::new();
            for _ in (min + 1)..max {
                hook_to_end.push(at);
                let next = add_nfa(&mut nfa, middle.clone());
                nfa[at.end].add_epsilon(next.start);
            }
            let end = new_epsilon(&mut nfa, Vec::new());

            for range in hook_to_end {
                nfa[range.end].add_epsilon(end);
            }
        }
    }
    nfa
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_epsilon() {
        let mut node = Epsilon(Vec::new());
        node.add_epsilon(1);
        node.add_epsilon(10);
        assert_eq!(node, Epsilon(vec![1, 10]));
    }

    #[test]
    fn test_add_nfa() {
        let mut first = vec![Character(b'a', 1), Epsilon(Vec::new())];
        let second = vec![Character(b'b', 1), Epsilon(vec![0, 1])];
        let range = add_nfa(&mut first, second);
        assert_eq!(
            first,
            vec![
                Character(b'a', 1),
                Epsilon(Vec::new()),
                Character(b'b', 3),
                Epsilon(vec![2, 3])
            ]
        );
        assert_eq!(range, Range { start: 2, end: 3 });
    }
}

