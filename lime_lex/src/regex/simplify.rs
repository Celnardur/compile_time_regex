use crate::Error;
use crate::regex::RegexToken::*;
use std::collections::HashSet;


/// turns digits into numbers and checks for matching curly brackets
/// These two happen at the same time because numbers should 
/// only appear in {} and get turned into characters Outside them.
fn simpilify_digits(regex: &[RegexToken]) -> Result<Vec<RegexToken>>, Error> {
    let mut tokens = Vec::new();
    let mut regex: Vec<RegexToken> = regex.iter().cloned().rev().collect();

    let mut inCurly = false;
    while let Some(t) = regex.pop() {
        match t {
            RCurly => {
                if inCurly {
                    inCurly = false;
                    tokens.push(t);
                } else {
                    return Err(Error::new("Mismatched }"));
                }
            },
            LCurly => {
                if inCurly {
                    return Err(Error::new("Mismatched {"));
                } else {
                    inCurly = true;
                    tokens.push(t);
                }
            },
            Digit(d) => {
                if inCurly {
                    let mut number = d;
                }
            },
            _ => (),
        }
    }

    Ok(tokens)
}
