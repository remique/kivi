#[derive(PartialEq, Debug)]
pub enum KeywordType {
    Select,
    Insert,
}

#[derive(PartialEq, Debug)]
pub enum TokenType {
    Keyword(KeywordType),
    Identifier(String),

    PlusSign,
    MinusSign,
    EOF,
}

impl TokenType {
    // TODO: Actually we can do TryFrom
    pub fn get_keyword_type(input: &str) -> Option<KeywordType> {
        match input {
            "select" => Some(KeywordType::Select),
            "insert" => Some(KeywordType::Insert),
            _ => None,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub position: usize,
}

impl Token {
    pub fn new(token_type: TokenType, position: usize) -> Self {
        Self {
            token_type,
            position,
        }
    }
}
