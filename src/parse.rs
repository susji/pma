use std::collections::VecDeque;
use std::error;
use std::fmt;

use crate::sexpr::SExpr;
use crate::tokens::Token;
use crate::tokens::Tokens;

#[derive(Debug, Clone)]
pub struct ParseError {
    msg: String,
    lineno: u64,
    col: u64,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}: sexpr error: {}", self.lineno, self.col, self.msg)
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl ParseError {
    pub fn new(msg: &str, lineno: u64, col: u64) -> ParseError {
        ParseError {
            msg: msg.to_string(),
            lineno,
            col,
        }
    }
}

fn list(toks: &mut Tokens) -> Result<SExpr, ParseError> {
    let mut members: Vec<SExpr> = Vec::new();
    let mut end = false;
    while let Some(cur) = toks.pop() {
        match cur {
            (Token::LParen, _, _) => {
                members.push(list(toks)?);
            }
            (Token::RParen, _, _) => {
                end = true;
                break;
            }
            (Token::Str(s), _, _) => members.push(SExpr::Str(s)),
            (Token::Id(i), _, _) => members.push(SExpr::Id(i)),
        }
    }
    if !end {
        return Err(ParseError::new(
            "abrupt end of list",
            toks.lineno(),
            toks.col(),
        ));
    }
    Ok(SExpr::List(members))
}

pub fn parse(mut toks: Tokens) -> Result<VecDeque<SExpr>, ParseError> {
    let mut sexprs: VecDeque<SExpr> = VecDeque::new();
    while let Some(cur) = toks.peek() {
        match cur {
            Token::LParen => {
                toks.pop();
                sexprs.push_back(list(&mut toks)?)
            }
            _ => return Err(ParseError::new("expecting '('", toks.lineno(), toks.col())),
        }
    }
    Ok(sexprs)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! new_toks {
        ($($tok:expr),*) => {{
            let mut _toks = Tokens::new();
            $(_toks.push($tok, 0, 0);)*
            _toks
        }}
    }

    #[test]
    fn test_parse_basic() {
        let toks = new_toks!(Token::LParen, Token::Id("id".to_string()), Token::RParen);
        let mut sexprs = parse(toks).unwrap();
        assert_eq!(1, sexprs.len());
        assert_eq!(
            SExpr::List(vec!(SExpr::Id("id".to_string()))),
            sexprs.pop_front().unwrap()
        );
    }

    #[test]
    fn test_parse_two_exprs() {
        let toks = new_toks!(
            Token::LParen,
            Token::Id("one".to_string()),
            Token::RParen,
            Token::LParen,
            Token::Id("two".to_string()),
            Token::RParen
        );
        let mut sexprs = parse(toks).unwrap();
        assert_eq!(2, sexprs.len());
        assert_eq!(
            SExpr::List(vec!(SExpr::Id("one".to_string()))),
            sexprs.pop_front().unwrap()
        );
        assert_eq!(
            SExpr::List(vec!(SExpr::Id("two".to_string()))),
            sexprs.pop_front().unwrap()
        );
    }

    #[test]
    fn test_parse_nested() {
        let toks = new_toks!(
            Token::LParen,
            Token::Id("id".to_string()),
            Token::LParen,
            Token::Str("string".to_string()),
            Token::RParen,
            Token::RParen
        );
        let mut sexprs = parse(toks).unwrap();
        assert_eq!(1, sexprs.len());
        assert_eq!(
            SExpr::List(vec!(
                SExpr::Id("id".to_string()),
                SExpr::List(vec!(SExpr::Str("string".to_string())))
            )),
            sexprs.pop_front().unwrap()
        );
    }
}
