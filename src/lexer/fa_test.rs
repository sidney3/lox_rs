// Internal trait for describing finite automata
//
// The payoff for implementing this trait is a bunch of
// tests.
use super::regex::Regex;

pub trait FA<T> {
    fn make(token_defs: Vec<(T, Regex)>) -> Self;
    fn classify(&self, input: &str) -> Option<T>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenType {
    Literal,
    BetterLiteral,
    A,
    AStar,
    AB,
    Alt,
}

fn make<C>(grammar: Vec<(TokenType, &str)>) -> C
where
    C: FA<TokenType>,
{
    C::make(
        grammar
            .into_iter()
            .map(|(token, regex_str)| (token, Regex::make(regex_str).unwrap()))
            .collect(),
    )
}

pub fn run_tests<C>()
where
    C: FA<TokenType>,
{
    {
        let nfa = make::<C>(vec![(TokenType::Literal, ("abc"))]);

        assert!(nfa.classify("").is_none());
        assert!(nfa.classify("ab").is_none());
        assert!(nfa.classify("abcd").is_none());
        assert_eq!(nfa.classify("abc").unwrap(), TokenType::Literal);
    }

    {
        let nfa = make::<C>(vec![(TokenType::Literal, ("a*a"))]);

        assert!(nfa.classify("").is_none());
        assert!(nfa.classify("b").is_none());
        assert_eq!(nfa.classify("a").unwrap(), TokenType::Literal);
        assert_eq!(nfa.classify("aa").unwrap(), TokenType::Literal);
        assert_eq!(nfa.classify("aaaaaaaa").unwrap(), TokenType::Literal);
    }

    {
        let nfa = make::<C>(vec![
            (TokenType::BetterLiteral, ("[a-z]*")),
            (TokenType::Literal, ("[a-z]*")),
        ]);

        assert_eq!(
            nfa.classify("asbjasdflasdf").unwrap(),
            TokenType::BetterLiteral
        );
        assert_eq!(nfa.classify("slasdf").unwrap(), TokenType::BetterLiteral);
        assert!(nfa.classify("slasdf1alfs2").is_none());
    }

    {
        let nfa = make::<C>(vec![
            (TokenType::A, ("a")),
            (TokenType::AStar, ("a*")),
            (TokenType::AB, ("ab")),
            (TokenType::Alt, ("a|b")),
        ]);

        assert_eq!(nfa.classify("").unwrap(), TokenType::AStar);
        assert_eq!(nfa.classify("a").unwrap(), TokenType::A);
        assert_eq!(nfa.classify("ab").unwrap(), TokenType::AB);
    }
}
