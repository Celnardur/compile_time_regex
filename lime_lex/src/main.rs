use lime_lex::*;

fn main() {
    match lime_lex::regex::validate(r"a*?[^\0-}]") {
        Ok(r) => println!("{:?}", r),
        Err(e) => println!("{}", e),
    }

}
