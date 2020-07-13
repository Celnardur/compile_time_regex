use std::string::String;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

impl Pos {
    pub fn add(&self, right: &Pos) -> Pos {
        Pos { line: self.line + right.line, col: self.col + right.col }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Token {
    pub start: Pos,
    pub end: Pos,
    pub kind: TokenType,
}

#[derive(PartialEq, Clone, Debug)]
pub enum TokenType {
    // Data Types
    I64,
    U64,
    U8,
    F64,
    Bool,
    Char,
    Type,
    Enum,
    // Keywords
    Let,
    Mut,
    Function,
    Return,
    Yield,
    // Control Flow
    While,
    For,
    If,
    Else,
    Switch,
    // Grouping
    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    // Arithmetic operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
    // Bitwise operators
    LeftShift,
    RightShift,
    BitwiseAnd,
    BitwiseOr,
    // Comparisons
    LT,
    LE,
    EQ,
    GE,
    GT,
    Not,
    NE,
    And,
    Or,
    // Symbols
    Assign,
    Colon,
    Semicolon,
    Comma,
    Pound,
    Dot,
    // Literals
    Integer(i128),
    Double(f64),
    Character(char),
    StringLiteral(String),
    // Identifier
    Identifier(String),
    // comments
    LeftBlockComment,
    RightBlockComment,
    MultiLineComment(String),
    LineComment(String),
    Whitespace,
}
