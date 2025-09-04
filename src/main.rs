use machine::Machine;
use parser::parse;
use std::io::stdin;

mod machine;
mod parser;

/// Number type of the machine
type Num = f64;

type Result<T> = std::result::Result<T, String>;

fn main() {
    let mut machine = Machine::new();
    for line in stdin().lines().map_while(|l| l.ok()) {
        match parse(&line) {
            Ok(("", values)) => {
                for v in values {
                    if let Err(e) = machine.process(v) {
                        eprintln!("Error at {v:?}: {e} (stack was {:?})", machine.stack);
                        break;
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
    Value(Num),
    Add,
    Sub,
    Mul,
    Div,
    Print,
    Quit,
    Clear,
    Printall,
    Store,
    Load,
}

impl V {
    fn number(self) -> Result<Num> {
        match self {
            V::Value(v) => Ok(v),
            _ => Err(format!("Expected numeric value, got {self:?}")),
        }
    }
}
