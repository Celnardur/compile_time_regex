pub mod scan;
pub mod simplify;
pub mod parse;

use crate::Error;

pub fn validate(regex: &str) -> Result<parse::RAST, Error> {
    let tokens = scan::scan(regex)?;
    println!("{:?}", tokens);
    let simple = simplify::simpilfy(&tokens[..])?;
    println!("{:?}", simple);
    parse::parse(&simple[..])
}

