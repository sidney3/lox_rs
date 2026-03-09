use super::error::{Error, Result};
use smallvec::{SmallVec, smallvec};
use std::iter::Peekable;

#[derive(Debug, Eq, PartialEq)]
pub struct CharClass {
    ranges: SmallVec<[(char, char); 4]>,
}

impl CharClass {
    pub fn chars(&self) -> Vec<char> {
        self.ranges.iter().flat_map(|&(l, r)| l..=r).collect()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Regex {
    Or(Box<Regex>, Box<Regex>),
    Kleene(Box<Regex>),
    Concat(Box<Regex>, Box<Regex>),
    Literal(char),
    CharClass(CharClass),
}

impl Regex {
    fn or(lhs: Regex, rhs: Regex) -> Regex {
        Regex::Or(Box::new(lhs), Box::new(rhs))
    }

    fn concat(lhs: Regex, rhs: Regex) -> Regex {
        Regex::Concat(Box::new(lhs), Box::new(rhs))
    }

    fn kleene(r: Regex) -> Regex {
        Regex::Kleene(Box::new(r))
    }

    pub fn make(source: &str) -> Result<Self> {
        let mut parser = Parser::make(source)?;
        parser.parse()
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Token {
    LParen,
    RParen,
    Star,
    Pipe,
    Literal(char),
    LBrace,
    RBrace,
    Dash,
    // Plus(),
}

fn lex(program: &str) -> Result<Vec<Token>> {
    let mut iter = program.chars();
    let mut tokens = Vec::new();

    while let Some(c) = iter.next() {
        let token = match c {
            '(' => Token::LParen,
            ')' => Token::RParen,
            '*' => Token::Star,
            '|' => Token::Pipe,
            '\\' => match iter.next() {
                Some(c) => Token::Literal(c),
                None => return Err(Error::UnterminatedEscape),
            },
            '[' => Token::LBrace,
            ']' => Token::RBrace,
            '-' => Token::Dash,
            '+' => todo!(),
            c => Token::Literal(c),
        };
        tokens.push(token);
    }

    Ok(tokens)
}

// Regex follows the following grammar
//
//
// Regex -> Or
// Or -> Concat ('|' Or)*
// Concat -> Kleene+
// Kleene -> Atom ('*')*
// Atom -> '(' Regex ')' | Literal | CharClass
// CharClass -> '[' (Literal '-' Literal)+ ']'
//
//
struct Parser {
    iter: Peekable<std::vec::IntoIter<Token>>,
}

impl Parser {
    fn make(source: &str) -> Result<Self> {
        let tokens = lex(source)?;
        Ok(Parser {
            iter: tokens.into_iter().peekable(),
        })
    }

    fn parse(&mut self) -> Result<Regex> {
        self.parse_regex()
    }

    fn parse_regex(&mut self) -> Result<Regex> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Regex> {
        let mut lhs = self.parse_concat()?;

        while self.iter.next_if_eq(&Token::Pipe).is_some() {
            lhs = Regex::or(lhs, self.parse_or()?);
        }

        Ok(lhs)
    }

    fn parse_concat(&mut self) -> Result<Regex> {
        let lhs = self.parse_kleene()?;

        let res = match self.iter.peek() {
            Some(Token::LParen | Token::LBrace | Token::Literal(_)) => {
                Regex::concat(lhs, self.parse_concat()?)
            }
            _ => lhs,
        };

        Ok(res)
    }

    fn parse_kleene(&mut self) -> Result<Regex> {
        let mut kleened = self.parse_atom()?;

        while self.iter.next_if_eq(&Token::Star).is_some() {
            kleened = Regex::kleene(kleened);
        }

        Ok(kleened)
    }

    fn parse_atom(&mut self) -> Result<Regex> {
        let res = match self.iter.next() {
            Some(Token::LParen) => {
                let expr = self.parse_regex()?;
                self.expect(Token::RParen);
                expr
            }
            Some(Token::LBrace) => self.parse_class()?,
            Some(Token::Literal(c)) => Regex::Literal(c),
            _ => return Err(Error::UnterminatedRegex),
        };

        Ok(res)
    }

    fn parse_class(&mut self) -> Result<Regex> {
        let mut ranges = SmallVec::new();

        loop {
            match self.iter.next() {
                Some(Token::Literal(lhs)) => {
                    self.expect(Token::Dash);
                    let Some(Token::Literal(rhs)) = self.iter.next() else {
                        return Err(Error::MalformattedRange);
                    };
                    if !(lhs < rhs) {
                        return Err(Error::UnorderedRange(lhs, rhs));
                    }
                    ranges.push((lhs, rhs))
                }
                Some(Token::RBrace) => break,
                _ => return Err(Error::MalformattedRange),
            }
        }

        Ok(Regex::CharClass(CharClass { ranges }))
    }

    fn expect(&mut self, token: Token) -> () {
        assert_eq!(self.iter.next(), Some(token));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lex() {
        assert_eq!(lex("\\\\").unwrap(), vec![Token::Literal('\\')]);
        assert_eq!(
            lex("a|b").unwrap(),
            vec![Token::Literal('a'), Token::Pipe, Token::Literal('b')]
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(Regex::make("a").unwrap(), Regex::Literal('a'),);
        assert_eq!(
            Regex::make("a|b").unwrap(),
            Regex::or(Regex::Literal('a'), Regex::Literal('b')),
        );
        assert_eq!(Regex::make("(a)").unwrap(), Regex::Literal('a'));
        assert_eq!(
            Regex::make("(((a)))|(((b))*)").unwrap(),
            Regex::or(Regex::Literal('a'), Regex::kleene(Regex::Literal('b')))
        );
        assert_eq!(
            Regex::make("[a-zA-Z]*|b").unwrap(),
            Regex::or(
                Regex::kleene(Regex::CharClass(CharClass {
                    ranges: smallvec![('a', 'z'), ('A', 'Z'),]
                })),
                Regex::Literal('b'),
            )
        );
        assert_eq!(
            Regex::make("abc").unwrap(),
            Regex::concat(
                Regex::Literal('a'),
                Regex::concat(Regex::Literal('b'), Regex::Literal('c'),),
            )
        )
    }
}
