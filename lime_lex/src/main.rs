use lime_lex::*;

fn main() {
    let test = lime_lex::regex::scan_regex(r"a{4,5}");
    println!("{:?}", test);
}
