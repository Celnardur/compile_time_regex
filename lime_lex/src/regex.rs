pub mod nfa;
pub mod parse;
pub mod scan;
pub mod simplify;

use crate::Error;
use parse::UnaryOperation;
use parse::RAST;

pub fn get_rast(regex: &str) -> Result<parse::RAST, Error> {
    let tokens = scan::scan(regex)?;
    let simple = simplify::simpilfy(&tokens[..])?;
    let rast = parse::parse(&simple[..])?;
    check_rast(&rast)?;
    Ok(*rast)
}

pub fn get_nfa(regex: &str) -> Result<nfa::NFA, Error> {
    let tokens = scan::scan(regex)?;
    let simple = simplify::simpilfy(&tokens[..])?;
    let rast = parse::parse(&simple[..])?;
    check_rast(&rast)?;
    Ok(nfa::rast_to_nfa(&rast))
}

enum RegexType {
    Binary,
    Unary,
    Atomic,
}

fn check_rast(regex: &RAST) -> Result<RegexType, Error> {
    match regex {
        RAST::Binary(left, right, _) => {
            check_rast(&left)?;
            check_rast(&right)?;
            Ok(RegexType::Binary)
        }
        RAST::Unary(left, op) => {
            match op {
                UnaryOperation::MinMax(min, max) => {
                    if min >= max {
                        return Err(Error::new(
                            "In {min,max} operator, min should be less than max",
                        ));
                    }
                }
                UnaryOperation::Times(times) => {
                    if *times == 0 {
                        return Err(Error::new(
                            "In {times} operator, times should be greater than zero",
                        ));
                    }
                }
                _ => (),
            }
            let left = check_rast(&left)?;
            match left {
                RegexType::Unary => Err(Error::new("Cannot have two unary operations in a row")),
                _ => Ok(RegexType::Unary),
            }
        }
        RAST::Atomic(_) => Ok(RegexType::Atomic),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn adj_unary() {
        let regex = "a*+";
        let regex = crate::regex::get_rast(regex);
        assert_eq!(
            regex,
            Err(Error::new("Regex stoped parsing before the end"))
        );

        let regex = "(a*)+";
        let regex = crate::regex::get_rast(regex);
        assert_eq!(
            regex,
            Err(Error::new("Cannot have two unary operations in a row"))
        );
    }

    #[test]
    fn bad_times_min_max() {
        let regex = "a{2,1}";
        let regex = crate::regex::get_rast(regex);
        assert_eq!(
            regex,
            Err(Error::new(
                "In {min,max} operator, min should be less than max"
            ))
        );

        let regex = "a{0}";
        let regex = crate::regex::get_rast(regex);
        assert_eq!(
            regex,
            Err(Error::new(
                "In {times} operator, times should be greater than zero"
            ))
        );
    }
}
