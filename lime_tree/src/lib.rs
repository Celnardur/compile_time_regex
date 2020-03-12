pub mod pointer;
pub mod binary_tree;

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

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lime_tree/{}", self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
