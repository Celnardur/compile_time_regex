fn main() {
    match lime_lex::regex::get_rast(r"aa") {
        Ok(r) => println!("{:?}", r),
        Err(e) => println!("{}", e),
    }

}
