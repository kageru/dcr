use crate::{Num, Result, V, V::*};
use std::ops;

const STACK_EMPTY: &str = "not enough elements on the stack";
const NUM_REGISTERS: usize = 256;

pub struct Machine {
    pub stack: Vec<V>,
    registers: [Num; NUM_REGISTERS],
}

impl Machine {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            registers: [0.0; _],
        }
    }

    pub fn process(&mut self, v: V) -> Result<()> {
        dbg!(&v, &self.stack);
        self.process2::<false>(v)
    }

    fn process2<const APPLY: bool>(&mut self, v: V) -> Result<()> {
        Ok(match v {
            v @ Value(_) => self.stack.push(v),
            v @ (Fn1(_, _) | Fn(_)) if !APPLY => self.stack.push(v),

            Apply => {
                let o = self.pop()?;
                self.process2::<true>(o)?
            }
            Fn(o) => self.process(*o)?,
            Curry => {
                let vals = self.popn()?;
                let [Fn1(fun, None), arg] = vals else {
                    return Err(format!("Expected a function and an argument, got {vals:?}"));
                };
                self.push(Fn1(fun, Some(Box::new(arg))));
            }
            Fn1(o, None) => self.process(*o)?,
            Fn1(o, Some(v)) => {
                self.push(*v);
                self.process(*o)?;
            }

            Add => self.binop(ops::Add::add)?,
            Sub => self.binop(ops::Sub::sub)?,
            Mul => self.binop(ops::Mul::mul)?,
            Div => self.binop(ops::Div::div)?,

            Store => {
                let [value, addr] = self.popn()?;
                let addr = addr.int()?;
                *self.reg(addr)? = value.number()?;
            }
            Load => {
                let addr = self.pop()?.int()?;
                let value = *self.reg(addr)?;
                self.push(Value(value));
            }
            Stacksize => self.stack.push(Value(self.stack.len() as f64)),
            Repeat => {
                let [v, repetitions] = self.popn()?;
                for _ in 0..repetitions.int()? {
                    self.process2::<true>(v.clone())?;
                }
            }
            Clear => self.stack.clear(),

            Print => println!("{:?}", self.pop()?),
            Printall => println!("{:?}", self.stack),
            Quit => std::process::exit(0),
        })
    }

    fn binop<F: FnOnce(Num, Num) -> Num>(&mut self, f: F) -> Result<()> {
        let vals = self.popn()?;
        let [Value(a), Value(b)] = vals else {
            return Err(format!("Expected 2 numeric values, got {vals:?}"));
        };
        Ok(self.stack.push(Value(f(a, b))))
    }

    fn popn<const N: usize>(&mut self) -> Result<[V; N]> {
        // Checking first rather than `pop()?` because we don’t want to pop at all if there aren’t enough values.
        if self.stack.len() < N {
            return Err(STACK_EMPTY.to_owned());
        }
        let mut out = [const { Value(0.0) }; N];
        for i in (0..N).rev() {
            out[i] = self.stack.pop().unwrap();
        }
        Ok(out)
    }

    fn pop(&mut self) -> Result<V> {
        Ok(self.popn::<1>()?[0].clone())
    }

    fn push(&mut self, v: V) {
        self.stack.push(v);
    }

    fn reg(&mut self, i: usize) -> Result<&mut Num> {
        self.registers
            .get_mut(i)
            .ok_or_else(|| format!("Register {i} out of range"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn evaluation() {
        for (raw, expectation) in [
            ("1 2+3-", vec![Value(0.0)]),
            ("40 2+6/7*", vec![Value(49.0)]),
            ("5 2/3+3", vec![Value(5.5), Value(3.0)]),
            ("2 0s0l", vec![Value(2.0)]),
            ("10 0s 20 1s c 1l 1l + 0l -", vec![Value(30.0)]),
            // delayed application
            ("1 1 \\+ $", vec![Value(2.0)]),
            // partial application
            ("1 \\+ 1 1 + < $", vec![Value(3.0)]),
            // Calculate the average of [1, 2, 3, 4] using repeat and stack size commands
            ("1 2 3 4 S0s \\+ S2-r 0l /", vec![Value(2.5)]),
        ] {
            let input = parse(raw).expect("parsing failed").1;
            let mut machine = Machine::new();
            for v in input {
                machine.process(v).expect("processing failed");
            }
            assert_eq!(machine.stack, expectation, "{raw}");
        }
    }
}
