use super::dfa::DFA;
use super::error::{Error, Result};
use super::nfa::NFA;
use super::regex::Regex;
use lasso::{Rodeo, Spur};
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;

pub struct Lexer<T> {
    dfa: DFA<T>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token<T> {
    pub lexeme: Spur,
    pub token_type: T,
    pub line: usize,
}

impl<T> Lexer<T>
where
    T: Hash + Copy + Clone + Eq + PartialEq + Display,
{
    pub fn make(tokens: Vec<(T, &str)>) -> Result<Self> {
        let regex_mappings = tokens
            .into_iter()
            .map(|(token, regex_str)| match Regex::make(regex_str) {
                Ok(regex) => Ok((token, regex)),
                Err(error) => {
                    println!("Failed to parse regex str {}", regex_str);
                    Err(error)
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let nfa = NFA::make(regex_mappings);

        Ok(Self {
            dfa: DFA::make(nfa),
        })
    }

    pub fn lex(&self, program: &str, mut rodeo: &mut Rodeo) -> Result<Vec<Token<T>>> {
        let mut cursor = Cursor::make(program);
        let mut out = Vec::new();

        while let Some(token) = self.parse_token(&mut rodeo, &mut cursor) {
            out.push(token);
        }

        if cursor.is_eof() {
            Ok(out)
        } else {
            Err(Error::NoMatchingToken {
                line: cursor.line(),
            })
        }
    }

    fn parse_token(&self, rodeo: &mut Rodeo, cursor: &mut Cursor) -> Option<Token<T>> {
        let mut curr_state = self.dfa.initial_state;
        let mut history = vec![curr_state];
        let start = cursor.mark();
        // we want the line _before_ we consumed the token, not after!
        let line = cursor.line();

        while let Some(c) = cursor.advance() {
            match self.dfa[curr_state].transitions.get(&c) {
                Some(&next_state) => {
                    history.push(next_state);
                    curr_state = next_state;
                }
                None => {
                    cursor.rollback();
                    break;
                }
            }
        }

        while let Some(curr_state) = history.pop() {
            match self.dfa.terminal_states.get(&curr_state) {
                Some(&token_type) => {
                    let lexeme = cursor.slice(start);

                    // We will never accept an empty token
                    if lexeme.is_empty() {
                        return None;
                    }
                    return Some(Token {
                        lexeme: rodeo.get_or_intern(lexeme),
                        token_type,
                        line,
                    });
                }
                None => {
                    // The very first node in the history
                    // doesn't correspond to a character
                    if !history.is_empty() {
                        cursor.rollback();
                    }
                }
            }
        }

        None
    }
}

struct Cursor<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    history: Vec<usize>,
}

impl<'a> Cursor<'a> {
    fn make(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            history: Vec::new(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.history.push(self.pos);
        self.pos += c.len_utf8();
        self.line += (c == '\n') as usize;
        Some(c)
    }

    fn rollback(&mut self) {
        assert!(!self.history.is_empty());

        self.pos = self.history.pop().unwrap();
        self.line -= (self.peek() == Some('\n')) as usize;
    }

    fn mark(&self) -> usize {
        self.pos
    }

    fn slice(&self, start: usize) -> &'a str {
        &self.input[start..self.pos]
    }

    fn line(&self) -> usize {
        self.line
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    enum TokenT {
        Literal,
        Struct,
        Whitespace,
    }

    impl Display for TokenT {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                TokenT::Literal => write!(f, "Literal"),
                TokenT::Struct => write!(f, "Struct"),
                TokenT::Whitespace => write!(f, "Whitespace"),
            }
        }
    }

    #[test]
    fn test_literal_precedence() {
        let lexer = Lexer::make(vec![
            (TokenT::Whitespace, (" ")),
            (TokenT::Struct, ("struct")),
            (TokenT::Literal, ("[a-zA-Z]*")),
        ])
        .unwrap();

        let mut rodeo = Rodeo::default();

        let mut spur = |s| rodeo.get_or_intern(s);

        let ws_token = Token {
            lexeme: spur(" "),
            token_type: TokenT::Whitespace,
            line: 1,
        };
        let struct_token = Token {
            lexeme: spur("struct"),
            token_type: TokenT::Struct,
            line: 1,
        };

        let expected_tokens = vec![
            struct_token,
            ws_token,
            Token {
                lexeme: spur("structa"),
                token_type: TokenT::Literal,
                line: 1,
            },
            ws_token,
            Token {
                lexeme: spur("structs"),
                token_type: TokenT::Literal,
                line: 1,
            },
            ws_token,
            Token {
                lexeme: spur("sstruct"),
                token_type: TokenT::Literal,
                line: 1,
            },
            ws_token,
            struct_token,
        ];

        assert_eq!(
            lexer
                .lex("struct structa structs sstruct struct", &mut rodeo)
                .unwrap(),
            expected_tokens
        )
    }
}
