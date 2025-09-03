use machine::Machine;
use parser::parse;
use std::io::stdin;

mod machine;
mod parser;

/// Number type of the machine
type N = f64;

fn main() {
    let mut machine = Machine::new();
    for line in stdin().lines().map_while(Result::ok) {
        match parse(&line) {
            Ok(("", values)) => {
                for v in values {
                    if let Err(e) = machine.process(v) {
                        eprintln!("Error at {v:?}: {e} (stack was {:?})", machine.stack)
                    }
                }
            }
            // Should the valid parts still be executed?
            Ok((rest, _)) => eprintln!("Input contained unparsable tokens: “{rest}”"),
            Err(_) => unreachable!(), // it actually is!
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum V {
    Value(N),
    Add,
    Sub,
    Mul,
    Div,
    Print,
    Quit,
    Clear,
    Printall,
}
