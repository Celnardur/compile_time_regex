pub mod nfa;
pub use nfa::NFA;


#[derive(Debug)]
pub struct Error {
    message: String, 
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
        }
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