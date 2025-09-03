use crate::{N, V, V::*};
use std::ops;

const STACK_EMPTY: &str = "not enough elements on the stack";
const NUM_REGISTERS: usize = 256;

pub struct Machine {
    pub stack: Vec<N>,
    registers: [N; NUM_REGISTERS],
}

impl Machine {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            registers: [0.0; _],
        }
    }

    pub fn process(&mut self, v: V) -> Result<(), String> {
        Ok(match v {
            Value(v) => self.stack.push(v),
            Add => self.binop(ops::Add::add)?,
            Sub => self.binop(ops::Sub::sub)?,
            Mul => self.binop(ops::Mul::mul)?,
            Div => self.binop(ops::Div::div)?,

            Store => {
                let [value, addr] = self.popn()?;
                let addr = addr.round() as usize;
                *self.reg(addr)? = value;
            }
            Load => {
                let addr = self.pop()?.round() as usize;
                let value = *self.reg(addr)?;
                self.push(value);
            }
            Clear => self.stack.clear(),

            Print => println!("{}", self.pop()?),
            Printall => println!("{:?}", self.stack),
            Quit => std::process::exit(0),
        })
    }

    fn binop<F: FnOnce(N, N) -> N>(&mut self, f: F) -> Result<(), String> {
        let [a, b] = self.popn()?;
        Ok(self.stack.push(f(a, b)))
    }

    fn popn<const N: usize>(&mut self) -> Result<[N; N], String> {
        // Checking first rather than `pop()?` because we don’t want to pop at all if there aren’t enough values.
        if self.stack.len() < N {
            return Err(STACK_EMPTY.to_owned());
        }
        let mut out = [0.0; N];
        for i in (0..N).rev() {
            out[i] = self.stack.pop().unwrap();
        }
        Ok(out)
    }

    fn pop(&mut self) -> Result<N, String> {
        Ok(self.popn::<1>()?[0])
    }

    fn push(&mut self, v: N) {
        self.stack.push(v);
    }

    fn reg(&mut self, i: usize) -> Result<&mut N, String> {
        self.registers
            .get_mut(i)
            .ok_or_else(|| format!("Register {i} out of range"))
    }
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
            ("2 0s0l", vec![2.0]),
            ("10 0s 20 1s c 1l 1l + 0l -", vec![30.0]),
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
