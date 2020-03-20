fn main() {
    match lime_lex::regex::get_rast(r"a?*") {
        Ok(r) => println!("{:?}", r),
        Err(e) => println!("{}", e),
    }

}
