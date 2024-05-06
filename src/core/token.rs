#[derive(PartialEq, Debug)]
pub enum KeywordType {
    Select,
    Insert,
    From,
    Where,
}

#[derive(PartialEq, Debug)]
pub enum TokenType {
    Keyword(KeywordType),
    Identifier(String),

    SqlString(String),

    PlusSign,
    MinusSign,
    StarSign,
    EqualSign,
    EOF,
}

impl TryFrom<&str> for KeywordType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value_case_insensitive = value.to_ascii_lowercase();

        match value_case_insensitive.as_ref() {
            "select" => Ok(KeywordType::Select),
            "insert" => Ok(KeywordType::Insert),
            "from" => Ok(KeywordType::From),
            "where" => Ok(KeywordType::Where),
            _ => Err("Not a Keyword"),
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
