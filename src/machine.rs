use crate::{N, V, V::*};
use std::ops;

const STACK_EMPTY: &str = "not enough elements on the stack";

pub struct Machine {
    pub stack: Vec<N>,
    registers: [N; 256],
}

impl Machine {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            registers: [0.0; 256],
        }
    }

    pub fn process(&mut self, v: V) -> Result<(), &'static str> {
        Ok(match v {
            Value(v) => self.stack.push(v),
            Add => binop(&mut self.stack, ops::Add::add)?,
            Sub => binop(&mut self.stack, ops::Sub::sub)?,
            Mul => binop(&mut self.stack, ops::Mul::mul)?,
            Div => binop(&mut self.stack, ops::Div::div)?,
            Print => println!("{}", self.stack.last().ok_or(STACK_EMPTY)?),
            Printall => println!("{:?}", self.stack),
            Clear => self.stack.clear(),
            Quit => std::process::exit(0),
        })
    }
}

fn binop<F: FnOnce(N, N) -> N>(stack: &mut Vec<N>, f: F) -> Result<(), &'static str> {
    let [a, b] = popn(stack)?;
    Ok(stack.push(f(a, b)))
}

fn popn<const N: usize>(v: &mut Vec<N>) -> Result<[N; N], &'static str> {
    // Checking first rather than `pop()?` because we don’t want to pop at all if there aren’t enough values.
    if v.len() < N {
        return Err(STACK_EMPTY);
    }
    let mut out = [0.0; N];
    for i in (0..N).rev() {
        out[i] = v.pop().unwrap();
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_test() {
        for (input, expectation) in [
            ("1 2+3-", vec![0.0]),
            ("40 2+6/7*", vec![49.0]),
            ("5 2/3+3", vec![5.5, 3.0]),
        ] {
            let input = crate::parser::parse(input).expect("parsing failed").1;
            let mut machine = Machine::new();
            for v in input {
                machine.process(v).expect("processing failed");
            }
            assert_eq!(machine.stack, expectation);
        }
    }
}
