use machine::Machine;
use parser::parse;
use std::{fmt, io::stdin};

mod machine;
mod parser;
mod stdlib;

/// Number type of the machine
type Num = f64;

type Result<T> = std::result::Result<T, String>;

fn main() {
    let mut machine = Machine::new();
    for line in stdin().lines().map_while(|l| l.ok()) {
        match parse(&line) {
            Ok(("", values)) => {
                for v in values {
                    if let Err(e) = machine.process(v.clone()) {
                        eprintln!("Error at {v}: {e}, stack was:\n");
                        machine.process(V::Printall).unwrap();
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

#[derive(Debug, Clone, PartialEq)]
enum V {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // User interaction
    Print,
    Printall,
    Quit,
    // Stack manipulation
    Value(Num),
    Stacksize,
    Clear,
    Repeat,
    Store,
    Load,
    // Partial application and function references
    Apply,
    Curry,
    Fun(Box<V>),
    // (Fn, Arg)
    Curried(Box<V>, Box<V>),
    Identifier(String),
    Compose,
    // (Fn, Fn), executed left to right
    Composed(Box<V>, Box<V>),
    // Logic and control flow
    LessThan,
    GreaterThan,
    Equal,
    Conditional,
}

impl V {
    fn number(self) -> Result<Num> {
        match self {
            V::Value(v) => Ok(v),
            _ => Err(format!("Expected numeric value, got {self:?}")),
        }
    }

    fn int(self) -> Result<usize> {
        Ok(self.number()?.round() as usize)
    }
}

impl fmt::Display for V {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use V::*;
        match self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Mul => write!(f, "*"),
            Div => write!(f, "/"),
            Mod => write!(f, "%"),
            Value(num) => write!(f, "{num}"),
            Store => write!(f, "s"),
            Load => write!(f, "l"),
            Apply => write!(f, "$"),
            Fun(fun) => write!(f, "{fun}"),
            Curried(fun, arg) => write!(f, "{arg} {fun}"),
            Identifier(ident) => write!(f, "{ident}"),
            Composed(a, b) => write!(f, "({a}, {b})"),
            LessThan => write!(f, "<"),
            GreaterThan => write!(f, ">"),
            Equal => write!(f, "="),
            Conditional => write!(f, "?"),
            default => write!(f, "{default:?}"),
        }
    }
}
