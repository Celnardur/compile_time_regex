use super::parse::BinaryOperation;
use super::parse::UnaryOperation;
use super::parse::RAST;
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
            let mut at = Range { start: 0, end: 0 };
            new_epsilon(&mut nfa, Vec::new());
            // start from one because at is already the first one added
            for _ in 0..min {
                let next = add_nfa(&mut nfa, middle.clone());
                nfa[at.end].add_epsilon(next.start);
                at = next;
            }
            let mut hook_to_end = Vec::new();
            for _ in min..max {
                hook_to_end.push(at);
                let next = add_nfa(&mut nfa, middle.clone());
                nfa[at.end].add_epsilon(next.start);
                at = next;
            }
            let end = at.end;

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
    use crate::Error;
    use rand::Rng;

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

    #[test]
    fn atomic() -> Result<(), Error> {
        let regex = "a";
        let nfa = crate::regex::get_nfa(regex)?;
        assert_eq!(nfa, vec![Character(b'a', 1), Epsilon(vec![])]);
        Ok(())
    }

    #[test]
    fn binary() -> Result<(), Error> {
        let regex = "ab";
        let nfa = crate::regex::get_nfa(regex)?;
        assert_eq!(
            nfa,
            vec![
                Character(b'a', 1),
                Epsilon(vec![2]),
                Character(b'b', 3),
                Epsilon(vec![])
            ]
        );

        let regex = "a|b";
        let nfa = crate::regex::get_nfa(regex)?;
        assert_eq!(
            nfa,
            vec![
                Epsilon(vec![1, 3]),
                Character(b'a', 2),
                Epsilon(vec![5]),
                Character(b'b', 4),
                Epsilon(vec![5]),
                Epsilon(vec![])
            ]
        );
        Ok(())
    }

    #[test]
    fn unary_kleen_closure() -> Result<(), Error> {
        let regex = "a*";
        let nfa = crate::regex::get_nfa(regex)?;
        assert_eq!(
            nfa,
            vec![
                Epsilon(vec![1, 3]),
                Character(b'a', 2),
                Epsilon(vec![3]),
                Epsilon(vec![0])
            ]
        );
        Ok(())
    }

    #[test]
    fn unary_plus() -> Result<(), Error> {
        let regex = "a+";
        let nfa = crate::regex::get_nfa(regex)?;
        assert_eq!(
            nfa,
            vec![
                Character(b'a', 1),
                Epsilon(vec![2]),
                Epsilon(vec![3, 5]),
                Character(b'a', 4),
                Epsilon(vec![5]),
                Epsilon(vec![2])
            ]
        );
        Ok(())
    }

    #[test]
    fn unary_question() -> Result<(), Error> {
        let regex = "a?";
        let nfa = crate::regex::get_nfa(regex)?;
        assert_eq!(
            nfa,
            vec![
                Epsilon(vec![1, 3]),
                Character(b'a', 2),
                Epsilon(vec![3]),
                Epsilon(vec![])
            ]
        );
        Ok(())
    }

    #[test]
    fn unary_times() -> Result<(), Error> {
        let regex = "a{3}";
        let nfa = crate::regex::get_nfa(regex)?;
        assert_eq!(
            nfa,
            vec![
                Character(b'a', 1),
                Epsilon(vec![2]),
                Character(b'a', 3),
                Epsilon(vec![4]),
                Character(b'a', 5),
                Epsilon(vec![]),
            ]
        );
        Ok(())
    }

    #[test]
    fn unary_min_max() -> Result<(), Error> {
        let regex = "a{2,4}";
        let nfa = crate::regex::get_nfa(regex)?;
        assert_eq!(
            nfa,
            vec![
                Epsilon(vec![1]),
                Character(b'a', 2),
                Epsilon(vec![3]),
                Character(b'a', 4),
                Epsilon(vec![5, 8]),
                Character(b'a', 6),
                Epsilon(vec![7, 8]),
                Character(b'a', 8),
                Epsilon(vec![]),
            ]
        );

        let regex = "a{0,3}";
        let nfa = crate::regex::get_nfa(regex)?;
        assert_eq!(
            nfa,
            vec![
                Epsilon(vec![1, 6]),
                Character(b'a', 2),
                Epsilon(vec![3, 6]),
                Character(b'a', 4),
                Epsilon(vec![5, 6]),
                Character(b'a', 6),
                Epsilon(vec![]),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_combo() -> Result<(), Error> {
        let regex = "a(b|c)*";
        let nfa = crate::regex::get_nfa(regex)?;
        assert_eq!(
            nfa,
            vec![
                Character(b'a', 1),
                Epsilon(vec![2]),
                Epsilon(vec![3, 9]),
                Epsilon(vec![4, 6]),
                Character(b'b', 5),
                Epsilon(vec![8]),
                Character(b'c', 7),
                Epsilon(vec![8]),
                Epsilon(vec![9]),
                Epsilon(vec![2]),
            ]
        );
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
            crate::regex::get_nfa(&regex);
        }
    }
}
