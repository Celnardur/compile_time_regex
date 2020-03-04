pub mod scan;
pub mod simplify;
pub mod parse;

use crate::Error;

pub fn get_rast(regex: &str) -> Result<parse::RAST, Error> {
    let tokens = scan::scan(regex)?;
    let simple = simplify::simpilfy(&tokens[..])?;
    parse::parse(&simple[..])
}

