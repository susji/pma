use crate::recipe::Recipe;
use crate::recipe::Thing;
use crate::sexpr::SExpr;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct SyntaxError {
    msg: String,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "syntax error: {}", self.msg)
    }
}

impl Error for SyntaxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl SyntaxError {
    fn new(msg: &str) -> SyntaxError {
        SyntaxError {
            msg: msg.to_string(),
        }
    }
}

fn eval_stridlist(sexpr: &[SExpr]) -> Result<Vec<Thing>, SyntaxError> {
    let mut ret = Vec::new();
    for subexpr in sexpr.iter() {
        match subexpr {
            SExpr::Str(s) => ret.push(Thing::Actual(s.to_string())),
            SExpr::Id(i) => ret.push(Thing::Pseudo(i.to_string())),
            _ => return Err(SyntaxError::new("not a string or identifier")),
        }
    }
    Ok(ret)
}

fn eval_strlist(sexpr: &[SExpr]) -> Result<Vec<String>, SyntaxError> {
    let mut ret = Vec::new();
    for subexpr in sexpr.iter() {
        match subexpr {
            SExpr::Str(s) => ret.push(s.to_string()),
            _ => return Err(SyntaxError::new("not a string")),
        }
    }
    Ok(ret)
}

fn eval_target(rec: &mut Recipe, sexpr: &[SExpr]) -> Result<(), SyntaxError> {
    if sexpr.len() != 4 {
        return Err(SyntaxError::new("target: expecting 4 list elements"));
    }
    let name = match &sexpr[1] {
        SExpr::Str(s) => Thing::Actual(s.to_string()),
        SExpr::Id(i) => Thing::Pseudo(i.to_string()),
        _ => return Err(SyntaxError::new("target: expecting target name")),
    };
    let deps = match &sexpr[2] {
        SExpr::List(l) => eval_stridlist(l)?,
        _ => return Err(SyntaxError::new("target: expecting a list of dependencies")),
    };
    let cmds = match &sexpr[3] {
        SExpr::List(l) => eval_strlist(l)?,
        _ => return Err(SyntaxError::new("target: expecting a list of commands")),
    };
    rec.add_rule(name, deps.into_iter(), cmds);
    Ok(())
}

fn eval_set(rec: &mut Recipe, sexpr: &[SExpr]) -> Result<(), SyntaxError> {
    if sexpr.len() != 3 {
        return Err(SyntaxError::new("set: expecting 2 list elements"));
    }
    let name = match &sexpr[1] {
        SExpr::Str(s) => s,
        _ => return Err(SyntaxError::new("set: expecting parameter name as string")),
    };
    let value = match &sexpr[2] {
        SExpr::Str(s) => s,
        _ => return Err(SyntaxError::new("set: expecting parameter value as string")),
    };
    match rec.set_var(name, value) {
        Ok(_) => Ok(()),
        Err(e) => Err(SyntaxError::new(format!("set: {:?}", e).as_ref())),
    }
}

fn eval_list(rec: &mut Recipe, sexpr: &[SExpr]) -> Result<(), SyntaxError> {
    // We two different "applications":
    //   1. set
    //   2. target
    if sexpr.is_empty() {
        return Err(SyntaxError::new("nil list"));
    }
    let id = match &sexpr[0] {
        SExpr::Id(_id) => _id,
        _ => return Err(SyntaxError::new("expecting identifier")),
    };
    match id.as_str() {
        "target" => eval_target(rec, &sexpr),
        "set" => eval_set(rec, &sexpr),
        _ => Err(SyntaxError::new("unrecognized command")),
    }
}

pub fn eval<T>(sexprs: T) -> Result<Recipe, SyntaxError>
where
    T: Iterator<Item = SExpr>,
{
    let mut rec = Recipe::new();
    for sexpr in sexprs {
        match sexpr {
            SExpr::List(l) => match eval_list(&mut rec, &l) {
                Ok(_) => (),
                Err(e) => return Err(e),
            },
            _ => return Err(SyntaxError::new("top-level expression not a list")),
        };
    }
    Ok(rec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_target() {
        let s = SExpr::List(vec![
            SExpr::Id("target".to_string()),
            SExpr::Str("name".to_string()),
            SExpr::List(vec![
                SExpr::Str("depstr".to_string()),
                SExpr::Id("depid".to_string()),
            ]),
            SExpr::List(vec![SExpr::Str("cmd".to_string())]),
        ]);
        eval(vec![s].into_iter()).unwrap();
    }
}
