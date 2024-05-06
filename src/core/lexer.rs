use crate::core::{
    error::Result,
    token::{KeywordType, Token, TokenType},
};

#[derive(Debug, PartialEq)]
pub struct Lexer {
    input: String,
    current_char: char,
    current_position: usize,
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

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(tok) = self.next_token() {
            tokens.push(tok);
        }

        tokens
    }

    fn skip_whitespace(&mut self) {
        if self.current_char.is_whitespace() {
            self.read_char();
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        let token = match self.current_char {
            '+' => Some(Token::new(TokenType::PlusSign, self.current_position)),
            '-' => Some(Token::new(TokenType::MinusSign, self.current_position)),
            '*' => Some(Token::new(TokenType::StarSign, self.current_position)),
            '=' => Some(Token::new(TokenType::EqualSign, self.current_position)),
            '\'' => {
                // Skip first character
                self.read_char();
                let starting_pos = self.current_position;
                let res = self.read_sql_string();

                if let Ok(sql_string) = res {
                    Some(Token::new(sql_string, starting_pos))
                } else {
                    None
                }
            }
            //_ if self.current_char.is_ascii_digit() => {
            //    //
            //}
            _ if self.current_char.is_ascii_alphabetic() => {
                let starting_pos = self.current_position;
                let res = self.read_string();

                Some(Token::new(res, starting_pos))
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
        while self.current_char.is_ascii_alphabetic() || self.current_char == '_' {
            let c = self.read_char();
            if let Some(val) = c {
                res.push(val);
            } else {
                break;
            }
        }

        let res_str: String = res.into_iter().collect();

        if let Some(keyword) = KeywordType::try_from(res_str.as_str()).ok() {
            return TokenType::Keyword(keyword);
        }

        TokenType::Identifier(res_str)
    }

    /// This function reads an SQL string. Currently we only accept `\'` as a proper string,
    /// as it is based on PostgreSQL dialect, but will probably add support for others as well
    /// in the future.
    fn read_sql_string(&mut self) -> Result<TokenType> {
        let mut res: Vec<char> = Vec::new();
        while self.current_char != '\'' || self.current_char != '\0' {
            let c = self.read_char();
            match c {
                Some(val) => {
                    if val == '\'' {
                        break;
                    } else {
                        res.push(val);
                    }
                }
                None => break,
            }

            // Here we will probably return Err
        }

        let res_str: String = res.into_iter().collect();

        Ok(TokenType::SqlString(res_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::token::KeywordType;

    #[test]
    fn test_read_char() {
        // TODO: Parse numbers
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
                position: 1
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
    fn test_basic_tokenize() {
        let mut l = Lexer::new("+- hellou").unwrap();

        let result = vec![
            Token {
                token_type: TokenType::PlusSign,
                position: 0,
            },
            Token {
                token_type: TokenType::MinusSign,
                position: 1,
            },
            Token {
                token_type: TokenType::Identifier("hellou".to_string()),
                position: 3,
            },
        ];

        assert_eq!(l.tokenize(), result);
    }

    #[test]
    fn test_case_insensitive_input() {
        let mut l = Lexer::new("SeLeCt * fRoM").unwrap();

        let result = vec![
            Token {
                token_type: TokenType::Keyword(KeywordType::Select),
                position: 0,
            },
            Token {
                token_type: TokenType::StarSign,
                position: 7,
            },
            Token {
                token_type: TokenType::Keyword(KeywordType::From),
                position: 9,
            },
        ];

        assert_eq!(l.tokenize(), result);
    }

    #[test]
    fn test_string() {
        let mut l = Lexer::new("hello world").unwrap();
        assert_eq!(
            Some(Token {
                token_type: TokenType::Identifier("hello".to_string()),
                position: 0
            }),
            l.next_token(),
        );
        assert_eq!(
            Some(Token {
                token_type: TokenType::Identifier("world".to_string()),
                position: 6
            }),
            l.next_token(),
        );
        assert_eq!(None, l.next_token());
    }

    #[test]
    fn test_keyword() {
        let mut l = Lexer::new("select world").unwrap();
        assert_eq!(
            Some(Token {
                token_type: TokenType::Keyword(KeywordType::Select),
                position: 0
            }),
            l.next_token(),
        );
        assert_eq!(
            Some(Token {
                token_type: TokenType::Identifier("world".to_string()),
                position: 7
            }),
            l.next_token(),
        );
    }

    #[test]
    fn test_whole_query() {
        let mut l = Lexer::new("SeLect * fRom my_table WheRe my_param = 'ayaya'").unwrap();

        let result = vec![
            Token {
                token_type: TokenType::Keyword(KeywordType::Select),
                position: 0,
            },
            Token {
                token_type: TokenType::StarSign,
                position: 7,
            },
            Token {
                token_type: TokenType::Keyword(KeywordType::From),
                position: 9,
            },
            Token {
                token_type: TokenType::Identifier("my_table".to_string()),
                position: 14,
            },
            Token {
                token_type: TokenType::Keyword(KeywordType::Where),
                position: 23,
            },
            Token {
                token_type: TokenType::Identifier("my_param".to_string()),
                position: 29,
            },
            Token {
                token_type: TokenType::EqualSign,
                position: 38,
            },
            Token {
                token_type: TokenType::SqlString("ayaya".to_string()),
                position: 41,
            },
        ];

        assert_eq!(l.tokenize(), result);
    }
}
