use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::graph;
use crate::graph::GraphIndex;
use crate::Verbosity;

lazy_static! {
    static ref RE_VAR: Regex =
        Regex::new(r#"^(?P<first>\$[_a-zA-Z)]+)|[^\$](?P<second>\$[_a-zA-Z)]+)"#).unwrap();
}

type RunFunction = Box<dyn Fn(&Recipe, &String) -> bool>;
type RegenFunction = Box<dyn Fn(&Thing, &Thing) -> Result<bool, String>>;
type MarkMemory = HashSet<GraphIndex>;

#[derive(Debug, PartialEq)]
pub enum SearchResult {
    Cancelled,
    Ok,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Thing {
    Actual(String),
    Pseudo(String),
}

#[derive(Debug)]
pub struct Recipe {
    depgraph: graph::DAG<Thing>,
    inverse: HashMap<graph::GraphIndex, Thing>,
    rules: HashMap<Thing, graph::GraphIndex>,
    cmds: HashMap<Thing, Vec<String>>,
    vars: HashMap<String, String>,
    first: Option<Thing>,
    v: Verbosity,
}

impl Default for Recipe {
    fn default() -> Self {
        Self::new()
    }
}

impl Recipe {
    pub fn new() -> Recipe {
        Recipe {
            depgraph: graph::DAG::new(),
            inverse: HashMap::new(),
            rules: HashMap::new(),
            vars: HashMap::new(),
            cmds: HashMap::new(),
            first: None,
            v: Verbosity::Minimal,
        }
    }

    pub fn add_rule<T>(&mut self, thing: Thing, deps: T, cmds: Vec<String>)
    where
        T: Iterator<Item = Thing>,
    {
        if self.first == None {
            self.first = Some(thing.clone());
        }
        // When we are adding a rule for a target, we might have seen it before
        // as a dependency. Thus we have to check if `thing` is already in our
        // book-keeping before blindly adding it.
        let i = if self.rules.contains_key(&thing) {
            *self.rules.get(&thing).unwrap()
        } else {
            let i = self.depgraph.node(thing.clone());
            self.rules.insert(thing.clone(), i);
            self.inverse.insert(i, thing.clone());
            i
        };
        self.cmds.insert(thing, cmds);
        // When inserting a new rule into the dependency graph, we have to make
        // sure all its dependencies are
        //
        //   a) already in the graph, or
        //   b) inserted.
        //
        for dep in deps {
            let di: graph::GraphIndex = if self.rules.contains_key(&dep) {
                *self.rules.get(&dep).unwrap()
            } else {
                let di = self.depgraph.node(dep.clone());
                self.rules.insert(dep.clone(), di);
                self.inverse.insert(di, dep);
                di
            };
            self.depgraph.connect(i, di);
        }
    }

    fn expand_vars(
        &self,
        overrides: Option<HashMap<String, String>>,
        msg: &str,
    ) -> Result<String, String> {
        condln!(self.v, Verbosity::Debug, "[v] expanding vars: {:?}", msg);
        let mut ret = msg.to_string();

        // We have to wrap our parameter/variable expansion into a restart loop,
        // because a single replacement invalidates the rest of the matches in
        // the iterator.
        'restart: loop {
            'iter: for m in RE_VAR.captures_iter(&ret) {
                condln!(self.v, Verbosity::Debug, "[v] match={:?}", m);
                let (name, start, end) = if let Some(var) = m.name("first") {
                    (&var.as_str()[1..], var.start(), var.end())
                } else if let Some(var) = m.name("second") {
                    (&var.as_str()[1..], var.start(), var.end())
                } else {
                    continue 'iter;
                };
                condln!(
                    self.v,
                    Verbosity::Debug,
                    "[v] match: '{}' ({}, {})",
                    name,
                    start,
                    end
                );
                let mut replaced = false;
                let mut rep = "";
                if let Some(vrep) = self.get_var(name) {
                    rep = vrep;
                    replaced = true;
                }
                if let Some(ref or) = overrides {
                    if let Some(orep) = or.get(name) {
                        rep = orep;
                        replaced = true;
                    }
                }
                if !replaced {
                    return Err(format!("Unrecognized parameter in expansion: {}", name));
                }

                ret.replace_range(start..end, rep);
                condln!(self.v, Verbosity::Debug, "[v] after replacement: {}", ret);
                continue 'restart;
            }
            break 'restart;
        }
        // The regex above meant that all double-dollars are left alone, so we
        // must handle them here. This has the semi-intentional side-effect of
        // supporting more general build scripts and building of shell commands.
        // See `examples/ex01.pma` and the "BUILD" parameter for an example.
        Ok(ret.replace("$$", "$"))
    }

    pub fn toposort(
        &self,
        memmark: &mut MarkMemory,
        target: GraphIndex,
        runner: &RunFunction,
        regener: &RegenFunction,
    ) -> SearchResult {
        // Has this node been visited before?
        if memmark.contains(&target) {
            return SearchResult::Ok;
        }
        memmark.insert(target);

        let thingtarget = self.inverse.get(&target).unwrap();
        let succ = self.depgraph.successors(target).unwrap();
        let mut regen = false;
        let mut nsucc = 0u64;
        for dep in succ.into_iter() {
            nsucc += 1;
            let thingdep = self.inverse.get(&dep).unwrap();
            condln!(
                self.v,
                Verbosity::Verbose,
                "[] target={:?}, dep={:?}",
                thingtarget,
                thingdep
            );
            match self.toposort(memmark, dep, runner, regener) {
                r @ SearchResult::Cancelled => return r,
                SearchResult::Ok => (),
            }
            match regener(thingtarget, thingdep) {
                Err(e) => {
                    condln!(self.v, Verbosity::Verbose, "[!] {:?}", e);
                    return SearchResult::Cancelled;
                }
                Ok(false) => {
                    condln!(self.v, Verbosity::Verbose, "[?] => target not out of date.");
                }
                Ok(true) => {
                    condln!(
                        self.v,
                        Verbosity::Verbose,
                        "[?] => regenerating {:?}",
                        thingtarget
                    );
                    regen = true;
                }
            }
        }
        if regen || nsucc == 0 {
            if let Some(cmds) = self.cmds.get(thingtarget) {
                for cmd in cmds.iter() {
                    let mut overrides: HashMap<String, String> = HashMap::new();
                    match thingtarget {
                        Thing::Actual(s) | Thing::Pseudo(s) => {
                            overrides.insert("TARGET".to_string(), s.to_string());
                            overrides.insert(
                                "DEPS".to_string(),
                                self.depgraph
                                    .successors(target)
                                    .unwrap()
                                    .into_iter()
                                    .map(|v| self.inverse.get(&v).unwrap())
                                    .map(|v| match v {
                                        Thing::Actual(s) | Thing::Pseudo(s) => s.to_string(),
                                    })
                                    .collect::<Vec<String>>()
                                    .join(" "),
                            );
                        }
                    }
                    let ecmd = match self.expand_vars(Some(overrides), cmd) {
                        Ok(e) => e,
                        Err(e) => {
                            eprintln!("Command expansion failed: {:?}", e);
                            return SearchResult::Cancelled;
                        }
                    };
                    if !runner(&self, &ecmd) {
                        return SearchResult::Cancelled;
                    }
                }
            }
        }
        SearchResult::Ok
    }

    pub fn evaluate(
        &self,
        thing: &Thing,
        runner: RunFunction,
        regener: RegenFunction,
    ) -> SearchResult {
        let ti = self.rules.get(thing).unwrap();
        let mut mm = MarkMemory::new();
        self.toposort(&mut mm, *ti, &runner, &regener)
    }

    pub fn set_var(&mut self, name: &str, val: &str) -> Result<(), String> {
        let eval = match self.expand_vars(None, &val) {
            Ok(e) => e,
            Err(e) => {
                return Err(format!("Parameter expansion failed: {:?}", e));
            }
        };
        self.vars.insert(name.to_string(), eval);
        Ok(())
    }

    pub fn get_var(&self, name: &str) -> Option<&String> {
        self.vars.get(name)
    }

    pub fn get_default(&self) -> Option<&Thing> {
        self.first.as_ref()
    }

    pub fn rule_exists(&self, thing: &Thing) -> bool {
        self.rules.contains_key(thing)
    }

    pub fn get_cmds(&self, thing: Thing) -> Option<&Vec<String>> {
        self.cmds.get(&thing)
    }

    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.v = verbosity;
    }

    pub fn get_verbosity(&self) -> Verbosity {
        self.v
    }
}
