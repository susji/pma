# pma

`pma` is a minimalistic builder based on s-expressions.

## Example use

`cargo` and a recent `rust` development environment are assumed. The
`pma` program expects build targets as command-line parameters. It
expects build rules from `stdin`.

	$ cd examples
	$ cat ex01.pma | cargo run clean all

If no targets are specified, the first encountered rule is evaluated.

## Grammar

`pma` files are always UTF-8-encoded.

	comment     = "#", no-lf, "\n" ;
	list        = "(", [ list-member ] , ")" ;
	list-member = list | atom ;
	atom        = identifier | string ;
	identifier  = { ( ascii-letter | "-" ) } ;
	string      = '"', { any-utf8-no-quote }, '"'

The above grammar defines the overall syntax. Exact semantics are specified
below.

## Functionality

`pma` builds targets if they are out-of-date with respect to their dependencies.
Build commands are strings passed to platform-dependent process execution. Build
commands undergo parameter expansion similarly to traditional `Makefile`s.

### Parameterizing build commands

The following target-specific parameter expansions are supported:

	$TARGET $DEPS

where `$TARGET` expands to the current target's filename and `$DEPS` expands to
a whitespace-delimited list of all dependencies.

### Parameter expansion

A valid parameter name is defined as follows:

	parameter  = "$", pchar,  { ( pchar | digit ) } ;
	pchar      = puppercase | plowercase | "_" ;
	puppercase = "A" | ... | "Z" ;
	plowercase = "a" | ... | "z" ;
	digit      = "0" | ... | "9" ;

To insert a plain `$` into a string, it must be escaped as `$$`.

### Declaring global parameters

Global parameters are defined like this:

	(set "<name>" "<value>")

The `<value>` expansion undergoes parameter expansion. As an example, consider
the following:

	(set "ARCH" "imaginary-arch")
	(set "CC" "$ARCH-cc")           # $CC => "imaginary-arch-cc"
	(set "LD" "$ARCH-ld")           # $LD => "imaginary-arch-ld"

### Dependency

A dependency is either an actual file or a pseudo-target. Pseudo-targets are
labelled with identifiers and actual files require strings.

### Declaring a target

A target declaration is used to generate `<target-filename>`. A target must have
one or more dependencies, which are also filenames. If there is a target rule
for a dependency, it will be evaluated recursively. After a target's
dependencies are found up-to-date without errors, its build commands will be
evaluated in the order of declaration. A failure in a single command will halt
the build process.

A target declaration is defined like this:

	(target
		"<target-filename>"
		(<str-or-id-1> ... <str-or-id-N)
		("<command-1>" ... "<command-N>"))


Target declarations to build a C program could look like this:

	(target "foo_util.o" ("foo_util.c") ("$CC -c -o $TARGET $DEPS"))
	(target "foo_main.o" ("foo_main.c" ("$CC -c -o $TARGET $DEPS"))
	(target "foo" ("foo_util.o" "foo_main.o") ("$LD -o $TARGET $DEPS"))

### Declaring a pseudo-target rule

Pseudo-targets are rules without any link to an actual file. They are always
considered out-of-date. A pseudo-target declaration is defined like this:

	(target
		<pseudo-target-id>
		(<str-or-id-1> ... <str-or-id-N)
		("<command-1>" ... "<command-N>"))

### Resolving dependencies

To build a target, all of its dependencies have to be evaluated in the correct
order. Each dependency is one of the following kinds and the respective action
will be taken:

* Pseudo-target; always regenerated
* Target with a rule; built if out of date
* Target without a rule; used if exists, error if nonexistent
