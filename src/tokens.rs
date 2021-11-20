use std::collections::VecDeque;

#[derive(Debug)]
pub struct Tokens {
    curlineno: u64,
    curcol: u64,
    lineno: VecDeque<u64>,
    col: VecDeque<u64>,
    toks: VecDeque<Token>,
}

impl Default for Tokens {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokens {
    pub fn new() -> Tokens {
        Tokens {
            curlineno: 0,
            curcol: 0,
            lineno: VecDeque::new(),
            col: VecDeque::new(),
            toks: VecDeque::new(),
        }
    }

    pub fn push(&mut self, tok: Token, lineno: u64, col: u64) {
        self.lineno.push_back(lineno);
        self.col.push_back(col);
        self.toks.push_back(tok);
    }

    pub fn pop(&mut self) -> Option<(Token, u64, u64)> {
        if self.toks.is_empty() {
            return None;
        }
        self.curlineno = self.lineno[0];
        self.curcol = self.col[0];
        Some((
            self.toks.pop_front().unwrap(),
            self.lineno.pop_front().unwrap(),
            self.col.pop_front().unwrap(),
        ))
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn peek(&self) -> Option<Token> {
        if self.toks.is_empty() {
            return None;
        }
        Some(self.toks[0].clone())
    }

    pub fn lineno(&self) -> u64 {
        self.curlineno
    }

    pub fn col(&self) -> u64 {
        self.curcol
    }

    pub fn len(&self) -> usize {
        self.toks.len()
    }
}

impl Iterator for Tokens {
    type Item = (Token, u64, u64);
    fn next(&mut self) -> Option<(Token, u64, u64)> {
        self.pop()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    LParen,
    Id(String),
    Str(String),
    RParen,
}
