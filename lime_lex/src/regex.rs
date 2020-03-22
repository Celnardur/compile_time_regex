pub mod parse;
pub mod scan;
pub mod simplify;
pub mod nfa;

use crate::Error;
use parse::RAST;

pub fn get_rast(regex: &str) -> Result<parse::RAST, Error> {
    let tokens = scan::scan(regex)?;
    let simple = simplify::simpilfy(&tokens[..])?;
    let rast = parse::parse(&simple[..])?;
    check_rast(&rast)?;
    Ok(*rast)
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
        RAST::Unary(left, _) => {
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
}
