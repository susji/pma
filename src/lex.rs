use std::error;
use std::fmt;

use crate::tokens::{Token, Tokens};

#[derive(Debug, Clone)]
pub struct LexError {
    msg: String,
    lineno: u64,
    col: u64,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}: lex error: {}", self.lineno, self.col, self.msg)
    }
}

impl error::Error for LexError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl LexError {
    pub fn new(msg: &str, lineno: u64, col: u64) -> LexError {
        LexError {
            msg: msg.to_string(),
            lineno,
            col,
        }
    }
}

fn lex_string(
    it: &mut std::iter::Peekable<std::str::Chars>,
    lineno: &mut u64,
    col: &mut u64,
) -> Result<String, LexError> {
    let mut res = String::new();
    let start = *col;
    let mut backslashed = false;
    let mut end = false;
    while let Some(&c) = it.peek() {
        *col += 1;
        match c {
            '\\' => {
                backslashed = !backslashed;
                res.push(c);
            }
            '"' => {
                if !backslashed {
                    end = true;
                } else {
                    // De-escape quote.
                    res.pop();
                    res.push(c);
                    backslashed = false;
                }
            }
            '\n' => {
                *lineno += 1;
                *col = 1;
                res.push(c);
            }
            _ => res.push(c),
        }
        it.next();
        if end {
            break;
        }
    }
    if !end {
        return Err(LexError::new("unterminated string", *lineno, start));
    }
    Ok(res)
}

pub fn lex(src: &str) -> Result<Tokens, LexError> {
    let mut res = Tokens::new();
    let mut it = src.chars().peekable();
    let mut lineno = 1;
    let mut col = 0;
    let mut fastforward = false;

    while let Some(&c) = it.peek() {
        it.next();
        col += 1;
        if fastforward {
            if c == '\n' {
                fastforward = false;
                lineno += 1;
            }
            continue;
        }
        match c {
            '(' => res.push(Token::LParen, lineno, col),
            ')' => res.push(Token::RParen, lineno, col),
            '\n' => {
                lineno += 1;
                col = 0;
            }
            ' ' | '\t' => (),
            '#' => fastforward = true,
            '-' | 'a'..='z' | 'A'..='Z' => {
                let mut id = c.to_string();
                let start = col;
                while let Some(&cc) = it.peek() {
                    match cc {
                        '-' | 'a'..='z' | 'A'..='Z' => {
                            id.push(cc);
                            col += 1;
                        }
                        _ => break,
                    }
                    it.next();
                }
                res.push(Token::Id(id), lineno, start);
            }
            '"' => {
                let start = col;
                match lex_string(&mut it, &mut lineno, &mut col) {
                    Ok(s) => res.push(Token::Str(s), lineno, start),
                    Err(e) => return Err(e),
                }
            }

            _ => return Err(LexError::new("unexpected character", lineno, col)),
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_string() {
        let src = r#"string" symbol"#;
        let mut lineno = 1;
        let mut col = 1;
        let mut it = src.chars().peekable();
        match lex_string(&mut it, &mut lineno, &mut col) {
            Ok(s) => {
                println!("result: {}", s);
                assert_eq!("string", s);
            }
            Err(e) => panic!("error: {}", e),
        };
    }

    #[test]
    fn test_lex_string_escaped() {
        let src = r#"\"string\"" symbol"#;
        let mut lineno = 1;
        let mut col = 1;
        let mut it = src.chars().peekable();
        match lex_string(&mut it, &mut lineno, &mut col) {
            Ok(s) => {
                println!("result: {}", s);
                assert_eq!(r#""string""#, s);
            }
            Err(e) => panic!("error: {}", e),
        };
    }

    #[test]
    fn test_lex_list() {
        let src = r#"(id-one id-two "str")"#;
        let res = match lex(src) {
            Ok(r) => r,
            Err(e) => panic!("error: {}", e),
        };
        println!("{:?}", res);
        assert_eq!(5, res.len());

        let toks: Vec<(Token, u64, u64)> = res.into_iter().collect();

        assert_eq!(Token::LParen, toks[0].0);
        assert_eq!(Token::Id("id-one".to_string()), toks[1].0);
        assert_eq!(Token::Id("id-two".to_string()), toks[2].0);
        assert_eq!(Token::Str("str".to_string()), toks[3].0);
        assert_eq!(Token::RParen, toks[4].0);
    }

    #[test]
    fn test_lex_comment() {
        let src = r###"# This is a comment.
(one two)"###;

        let res = match lex(src) {
            Ok(r) => r,
            Err(e) => panic!("error: {}", e),
        };

        let toks: Vec<(Token, u64, u64)> = res.into_iter().collect();

        assert_eq!(Token::LParen, toks[0].0);
        assert_eq!(Token::Id("one".to_string()), toks[1].0);
        assert_eq!(Token::Id("two".to_string()), toks[2].0);
        assert_eq!(Token::RParen, toks[3].0);
    }
}
