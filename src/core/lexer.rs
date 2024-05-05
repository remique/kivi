use crate::core::error::Result;

#[derive(Debug, PartialEq)]
pub struct Lexer {
    input: String,
    current_char: char,
    current_position: usize,
}

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

#[derive(PartialEq, Debug)]
pub struct Token {
    token_type: TokenType,
    position: usize,
}

impl Token {
    fn new(token_type: TokenType, position: usize) -> Self {
        Self {
            token_type,
            position,
        }
    }
}

impl Lexer {
    // TODO: Impl default
    pub fn new(input: &str) -> Result<Self> {
        let lexer = Lexer {
            input: input.into(),
            current_char: input.chars().next().unwrap(),
            current_position: 0,
        };

        Ok(lexer)
    }

    fn next_token(&mut self) -> Option<Token> {
        let token = match self.current_char {
            '+' => Some(Token::new(TokenType::PlusSign, 0)),
            '-' => Some(Token::new(TokenType::MinusSign, 0)),
            //_ if self.current_char.is_ascii_digit() => {
            //    //
            //}
            _ if self.current_char.is_ascii_alphabetic() => {
                let x = self.read_string();

                Some(Token::new(x, 0))
            }
            _ => None,
        };

        // Skip appropriate amount of characters
        self.read_char();

        token
    }

    /// This function reads a character, returns it and moves to the next position
    fn read_char(&mut self) -> Option<char> {
        let curr_char = self.current_char;
        if curr_char == '\0' {
            return None;
        }

        let next_position = self.current_position.saturating_add(1);

        if next_position >= self.input.len() {
            self.current_position = self.current_position.saturating_add(1);
            self.current_char = '\0';
        } else {
            self.current_position = self.current_position.saturating_add(1);
            self.current_char = self.input.as_bytes()[self.current_position] as char;
        }

        // Return current character
        Some(curr_char)
    }

    fn read_string(&mut self) -> TokenType {
        let mut res: Vec<char> = Vec::new();
        while self.current_char.is_ascii_alphabetic() {
            let c = self.read_char();
            if let Some(val) = c {
                res.push(val);
            } else {
                break;
            }
        }

        let res_str: String = res.into_iter().collect();

        TokenType::Identifier(res_str)

        // then we match on the string and check if its any of the reserved keywords
        // if its not then its identifier else its a keyword
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_char() {
        let mut l = Lexer::new("+-a 0.2").unwrap();

        assert_eq!(l.read_char(), Some('+'));
        assert_eq!(l.read_char(), Some('-'));
        assert_eq!(l.read_char(), Some('a'));
        assert_eq!(l.read_char(), Some(' '));
        assert_eq!(l.read_char(), Some('0'));
        assert_eq!(l.read_char(), Some('.'));
        assert_eq!(l.read_char(), Some('2'));
        assert_eq!(l.read_char(), None);
    }

    #[test]
    fn test_initial() {
        let mut l = Lexer::new("+-").unwrap();
        assert_eq!(
            l,
            Lexer {
                input: "+-".to_string(),
                current_char: '+',
                current_position: 0,
            }
        );
        assert_eq!(
            Some(Token {
                token_type: TokenType::PlusSign,
                position: 0
            }),
            l.next_token(),
        );
        assert_eq!(
            l,
            Lexer {
                input: "+-".to_string(),
                current_char: '-',
                current_position: 1,
            }
        );
        assert_eq!(
            Some(Token {
                token_type: TokenType::MinusSign,
                position: 0
            }),
            l.next_token(),
        );
        assert_eq!(
            l,
            Lexer {
                input: "+-".to_string(),
                current_char: '\0',
                current_position: 2,
            }
        );
    }

    #[test]
    fn test_string() {
        let mut l = Lexer::new("hello w").unwrap();
        assert_eq!(
            Some(Token {
                token_type: TokenType::Identifier("hello".to_string()),
                position: 0
            }),
            l.next_token(),
        );
        assert_eq!(
            Some(Token {
                token_type: TokenType::Identifier("w".to_string()),
                position: 0
            }),
            l.next_token(),
        );
        assert_eq!(None, l.next_token(),);
    }
}
