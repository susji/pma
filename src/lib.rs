#[derive(Debug, Eq, PartialEq, PartialOrd, Copy, Clone)]
pub enum Verbosity {
    Minimal,
    Verbose,
    Debug,
}

#[macro_export]
macro_rules! condln {
    ($value:expr, $cur:expr, $($tok:expr),*) => {
        if ($value >= $cur) {
            println!($($tok),*);
        }
    }
}

pub mod eval;
pub mod graph;
pub mod lex;
pub mod parse;
pub mod recipe;
pub mod sexpr;
pub mod tokens;
