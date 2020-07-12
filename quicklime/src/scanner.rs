use crate::token::*;
use TokenType::*;
use std::vec::Vec;
use std::error;

pub fn scan(code: Vec<char>) -> Result<Vec<Token>, Box<dyn error::Error>> {
    let mut on = Pos {
        line: 0,
        col: 0,
    };
    let mut index = 0;
    let mut tokens: Vec<Token> = Vec::new();

    while let Some((token, pos, length)) = parse_token(&code, index)? {
    }
    Ok(vec![])
}

pub fn parse_token(code: &Vec<char>, start_index: usize)
    -> Result<Option<(TokenType, Pos, usize)>, Box<dyn error::Error>> {
    if start_index >= code.len() {
        return Ok(None);
    }
    let code = &code[start_index..];

    // Identifiers and keywords
    if code[0].is_alphabetic() {
        let mut length = 1;
        while length < code.len() && code[length].is_alphanumeric(){
            length += 1;
        }

        let pos = Pos {line: 0, col: length};
        let id = code[..length].iter().collect::<String>();
        let id = id.as_str();
        // a keyword is just a special identifier
        let token = match id {
            "i64" => I64,
            "u64" => U64,
            "u8" => U8,
            "f64" => F64,
            "bool" => Bool,
            "char" => Char,
            "type" => Type,
            "enum" => Enum,
            "let" => Let,
            "mut" => Mut,
            "function" => Function,
            "return" => Return,
            "yield" => Yield,
            "while" => While,
            "for" => For,
            "if" => If,
            "else" => Else,
            _ => return Ok(Some((Identifier(id.to_owned()), pos, length))),
        };
        return Ok(Some((token, pos, length)));
    }

    // check for number literals
    if code[0].is_ascii_digit() {
        let mut length = 1;
        while length < code.len() && code[length].is_ascii_digit() {
            length += 1;
        }
        // double literal
        let token = if length < code.len() && code[length] == '.' {
            length += 1;
            while length < code.len() && code[length].is_ascii_digit() {
                length += 1;
            }
            // TODO: handle bad parses
            Double(code[..length].iter().collect::<String>().parse()?)
        } else {
            Integer(code[..length].iter().collect::<String>().parse()?)
        };

        return Ok(Some((token, Pos {col: length, line: 0}, length)))
    }

    Ok(None)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_token_test() {
        assert_eq!(
            parse_token(&"42".chars().collect(), 0).unwrap().unwrap(),
            (Integer(42), Pos {col: 2, line: 0}, 2)
        );

        assert_eq!(
            parse_token(&"asdf".chars().collect(), 0).unwrap().unwrap(),
            (Identifier("asdf".to_string()), Pos {col: 4, line: 0}, 4)
        );


    }
}