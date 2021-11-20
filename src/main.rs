use std::collections::VecDeque;
use std::process::{exit, Command};
use std::{
    env, fs,
    io::{self, Read},
};

use pma::eval::eval;
use pma::lex::lex;
use pma::parse::parse;
use pma::recipe::{Recipe, SearchResult, Thing};
use pma::{condln, Verbosity};

fn get_input() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn determine_targets(rec: &Recipe, names: VecDeque<String>) -> Option<VecDeque<Thing>> {
    let mut inerr = 0u64;
    let mut queue: VecDeque<Thing> = VecDeque::new();

    // The first evaluated rule is the default target. No default rule => no
    // valid targets.
    let def = rec.get_default();
    if def == None {
        eprintln!("No targets.");
        return None;
    }
    if names.is_empty() {
        queue.push_back(def.unwrap().clone());
        return Some(queue);
    }
    for name in names.into_iter() {
        let act = Thing::Actual(name.clone());
        let pse = Thing::Pseudo(name.clone());
        if rec.rule_exists(&act) {
            queue.push_back(act);
        } else if rec.rule_exists(&pse) {
            queue.push_back(pse);
        } else {
            eprintln!("Target does not exist: {:?}", name);
            inerr += 1;
        }
    }
    if inerr != 0 {
        return None;
    }
    Some(queue)
}

fn run_target(rec: &Recipe, thing: Thing) -> bool {
    condln!(
        rec.get_verbosity(),
        Verbosity::Verbose,
        "[] evaluating {:?}",
        thing
    );
    let res = rec.evaluate(
        &thing,
        Box::new(&|rec: &Recipe, cmd: &String| {
            condln!(rec.get_verbosity(), Verbosity::Verbose, "[cmd] {}", cmd);
            match Command::new("sh").arg("-c").arg(cmd.clone()).status() {
                Err(e) => {
                    eprintln!("Error when executing {}: {:?}", cmd, e);
                    false
                }
                Ok(ec) => ec.code() == Some(0),
            }
        }),
        Box::new(&|target: &Thing, dep: &Thing| -> Result<bool, String> {
            // We have four possibilities here:
            //
            //    1. Pseudo target, pseudo dependency => always regenerate
            //    2. Pseudo target, real dependency => always regenerate
            //    3. Actual target, pseudo dependency => always regenerate
            //    4. Actual target, actual dependency => compare modified times
            //
            match (target, dep) {
                (&Thing::Pseudo(_), _) | (_, &Thing::Pseudo(_)) => Ok(true),
                (&Thing::Actual(ref fn_target), &Thing::Actual(ref fn_dep)) => {
                    let mod_target = match fs::metadata(fn_target) {
                        Err(_) => return Ok(true), // target probably does not exist
                        Ok(md) => md.modified().unwrap(),
                    };
                    let mod_dep = match fs::metadata(fn_dep) {
                        Err(_) => return Ok(true), // dep probably does not exist
                        Ok(md) => md.modified().unwrap(),
                    };
                    Ok(mod_dep > mod_target)
                }
            }
        }),
    );
    match res {
        SearchResult::Cancelled => {
            println!("Build in error: {:?}", thing);
            false
        }
        SearchResult::Ok => {
            println!("Build successful: {:?}", thing);
            true
        }
    }
}

fn main() {
    let mut targets: VecDeque<String> = env::args().collect();

    let v = match std::env::var("PMA_VERBOSE") {
        Err(_) => Verbosity::Minimal,
        Ok(val) => match val.as_str() {
            "minimal" => Verbosity::Minimal,
            "verbose" => Verbosity::Verbose,
            "debug" => Verbosity::Debug,
            _ => Verbosity::Minimal,
        },
    };

    targets.pop_front();
    let input = match get_input() {
        Ok(s) => s,
        Err(s) => {
            eprintln!("unable to read input: {:?}", s);
            exit(1);
        }
    };
    let toks = match lex(&input) {
        Err(e) => {
            eprintln!("{:?}", e);
            exit(2)
        }
        Ok(t) => t,
    };
    let sexprs = match parse(toks) {
        Err(e) => {
            eprintln!("{:?}", e);
            exit(3);
        }
        Ok(s) => s,
    };
    let mut rec = match eval(sexprs.into_iter()) {
        Err(e) => {
            eprintln!("{:?}", e);
            exit(4);
        }
        Ok(s) => s,
    };

    condln!(v, Verbosity::Debug, "recipe: {:#?}", rec);
    rec.set_verbosity(v);

    let queue = determine_targets(&rec, targets);
    if queue == None {
        exit(5);
    }
    println!("Targets: {:?}.", queue);
    for cur in queue.unwrap().into_iter() {
        if !run_target(&rec, cur) {
            exit(6);
        }
    }
}
