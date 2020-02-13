pub mod nfa;
//pub use nfa::NFA;
pub mod regex;


#[derive(Debug)]
pub struct Error {
    message: String, 
    code: Option<String>, 
    line: u64,
    range: Option<(u32, u32)>,
}

impl Error {
    pub fn new_box(message: &str) -> Box<Error> {
        Box::new(Error {
            message: String::from(message),
        })
    }
    
    pub fn new(message: &str) -> Error {
        Error {
            message: String::from(message),
            code: None,
            line: 0,
            range: None,
        }
    }

    pub fn new_line(message: &str, code: &str, line: u64) -> Error {
        Error {
            message: String::from(message),
            code: Some(String::from(code)),
            line,
            range: None, 
        }
    }

    pub fn new_hl(message: &str, code: &str, line: u64, start: u32, end: u32) -> Error {
        Error {
            message: String::from(message),
            code: Some(String::from(code)),
            line,
            range: Some((start, end)),
        }
    }

    pub fn message(&self) -> &str {
        self.message()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)

    pub fn message(&self) -> &str {
        self.message()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
