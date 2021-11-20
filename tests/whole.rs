use pma::eval::eval;
use pma::lex::lex;
use pma::parse::parse;
use pma::recipe::Recipe;
use pma::recipe::Thing;
use pma::recipe::Thing::{Actual, Pseudo};

static mut STATE: u8 = 0;
static TEST: &str = r###"# This is an example. The default target will be "all".

(set "CC" "BUILD")

(target all ("foo") ("$$NOTVAR"))

(target
	"foo"
	("foo_main.o" "foo_util.o")
	("$CC FOO"))

(target
	"foo_main.o"
	("foo_main.c")
	("$CC FOO_MAIN"))

(target
	"foo_util.o"
	("foo_util.c")
	("$CC UTIL"))
"###;

#[test]
fn test_whole() {
    let toks = match lex(&TEST) {
        Err(e) => {
            panic!("{:?}", e);
        }
        Ok(t) => t,
    };
    let sexprs = match parse(toks) {
        Err(e) => {
            panic!("{:?}", e);
        }
        Ok(s) => s,
    };
    let rec = match eval(sexprs.into_iter()) {
        Err(e) => {
            panic!("{:?}", e);
        }
        Ok(s) => s,
    };

    rec.evaluate(
        &Thing::Pseudo("all".to_string()),
        Box::new(&|_: &Recipe, cmd: &String| {
            fn advance() -> u8 {
                unsafe {
                    let ret = STATE;
                    STATE += 1;
                    println!("state={} -> {}", ret, STATE);
                    return ret;
                }
            }
            println!("cmd: {:?}", cmd);
            match cmd.as_str() {
                "BUILD FOO_MAIN" => {
                    if advance() != 0 {
                        panic!("main rebuild not first");
                    }
                }
                "BUILD FOO" => {
                    if advance() != 1 {
                        panic!("foo rebuild not second");
                    }
                }
                "$NOTVAR" => {
                    if advance() != 2 {
                        panic!("all not third");
                    }
                }
                t @ _ => panic!("surprising cmd: {:?}", t),
            }
            true
        }),
        Box::new(&|target: &Thing, dep: &Thing| {
            println!("{:?} <- {:?}", target, dep);

            // "foo_main.o" is out of date, others are fresh. This should lead
            // into rebuilding of "foo" and foo_main.o".
            //
            // The way the state machine is encoded here is pretty ugly. A
            // prettier alternative would be to use a lookup table.
            match (target, dep) {
                (Actual(starget), Actual(sdep)) => match (starget.as_str(), sdep.as_str()) {
                    ("foo_main.o", "foo_main.c") => Ok(true),
                    ("foo", "foo_main.o") => unsafe { Ok(STATE == 1) },
                    (_, _) => Ok(false),
                },
                (Pseudo(starget), Actual(sdep)) => match (starget.as_str(), sdep.as_str()) {
                    ("all", "foo") => unsafe { Ok(STATE == 2) },
                    (_, _) => Ok(false),
                },
                (_, _) => Ok(false),
            }
        }),
    );
    unsafe {
        assert_eq!(3, STATE);
    }
}
