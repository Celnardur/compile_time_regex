fn main() {
    match lime_lex::regex::get_nfa(r"a(bc*d|ed)d*") {
        Ok(r) => println!("{:?}", r),
        Err(e) => println!("{}", e),
    }
}
