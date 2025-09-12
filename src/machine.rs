use crate::{Num, Result, V, V::*};
use std::{collections::HashMap, ops};

const STACK_EMPTY: &str = "not enough elements on the stack";
// Index 256 and above are for internal use.
const NUM_REGISTERS: usize = 266;

pub struct Machine {
    pub stack: Vec<V>,
    registers: [Num; NUM_REGISTERS],
    vars: HashMap<String, V>,
}

macro_rules! pop {
    ($machine:ident, $pattern:pat => $f:expr) => {{
        let vals = $machine.popn()?;
        let $pattern = vals else {
            let e = format!("Expected parameters, got {vals:?}");
            for val in vals {
                $machine.push(val);
            }
            return Err(e);
        };
        $f
    }};
}

impl Machine {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            registers: [0.0; _],
            vars: HashMap::new(),
        }
    }

    pub fn process(&mut self, v: V) -> Result<()> {
        self.process2::<false>(v)
    }

    fn process2<const APPLY: bool>(&mut self, v: V) -> Result<()> {
        Ok(match v {
            v @ Value(_) => self.stack.push(v),
            v @ (Fn1(_, _) | Fn2(_, _, _) | Fn(_) | Identifier(_)) if !APPLY => self.stack.push(v),

            Curry => match self.popn()? {
                [Fn1(f, None), v] => self.push(Fn1(f, Some(Box::new(v)))),
                [Fn2(f, a1, None), v] => self.push(Fn2(f, a1, Some(Box::new(v)))),
                [Fn2(f, None, a2), v] => self.push(Fn2(f, Some(Box::new(v)), a2)),
                [a, b] => {
                    let e = format!("Failed to curry {a:?} with {b:?}");
                    self.push(a);
                    self.push(b);
                    return Err(e);
                }
            },
            Apply => {
                let next = self.pop()?;
                self.process2::<true>(next)?
            }
            Identifier(ident) => match self.vars.get(&ident).cloned() {
                Some(v) => self.process2::<true>(v)?,
                None => return Err(format!("{ident} not found")),
            },
            Fn(o) => self.process(*o)?,
            Fn1(o, arg) => {
                if let Some(v) = arg {
                    self.push(*v);
                }
                self.process(*o)?;
            }
            Fn2(o, arg1, arg2) => {
                if let Some(v) = arg1 {
                    self.push(*v);
                }
                if let Some(v) = arg2 {
                    self.push(*v);
                }
                self.process(*o)?;
            }
            Compose => {
                let [a, b] = self.popn()?;
                self.push(Composed(Box::new(a), Box::new(b)));
            }

            Composed(a, b) => {
                self.process2::<true>(*a)?;
                self.process2::<true>(*b)?;
            }

            Add => self.binop(ops::Add::add)?,
            Sub => self.binop(ops::Sub::sub)?,
            Mul => self.binop(ops::Mul::mul)?,
            Div => self.binop(ops::Div::div)?,

            Store => {
                let [value, addr] = self.popn()?;
                if let Identifier(ident) = addr {
                    self.vars.insert(ident, value);
                } else {
                    let addr = addr.int()?;
                    *self.reg(addr)? = value.number()?;
                }
            }
            Load => {
                let addr = self.pop()?;
                let v = if let Identifier(ident) = addr {
                    self.vars
                        .get(&ident)
                        .cloned()
                        .ok_or_else(|| format!("{ident} not found"))?
                } else {
                    let addr = addr.int()?;
                    Value(*self.reg(addr)?)
                };
                self.push(v);
            }
            Stacksize => self.stack.push(Value(self.stack.len() as f64)),
            Repeat => {
                let [v, repetitions] = self.popn()?;
                for _ in 0..repetitions.int()? {
                    self.process2::<true>(v.clone())?;
                }
            }
            LessThan => pop!(self, [Value(a), Value(b)] => self.push(Value(f64::from(a < b)))),
            GreaterThan => pop!(self, [Value(a), Value(b)] => self.push(Value(f64::from(a > b)))),
            Equal => pop!(self, [Value(a), Value(b)] => self.push(Value(f64::from(a == b)))),
            Conditional => pop!(self, [Value(condition), a, b] => {
                self.push(
                    if condition == 0.0 {
                        b
                    } else {
                        a
                    }
                )
            }),
            Clear => self.stack.clear(),

            Print => println!("{:?}", self.pop()?),
            Printall => println!("{:?}", self.stack),
            Quit => std::process::exit(0),
        })
    }

    fn binop<F: FnOnce(Num, Num) -> Num>(&mut self, f: F) -> Result<()> {
        pop!(self, [Value(a), Value(b)] => Ok(self.stack.push(Value(f(a, b)))))
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
        let [v] = self.popn()?;
        Ok(v)
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
    use test_case::test_case;

    #[test_case("1 2+3-" => vec![Value(0.0)])]
    #[test_case("40 2+6/7*" => vec![Value(49.0)])]
    #[test_case("5 2/3+3" => vec![Value(5.5), Value(3.0)])]
    #[test_case("2 0s0l" => vec![Value(2.0)]; "storing a number in a register")]
    #[test_case("10 0s 20 1s c 1l 1l + 0l -" => vec![Value(30.0)])]
    #[test_case(r"1 1 \+ $" => vec![Value(2.0)]; "delayed application")]
    #[test_case(r"1 \+ 1 1 + @ $" => vec![Value(3.0)]; "partial application")]
    #[test_case(r"1 \+ 2 @" => vec![Value(1.0), Fn1(Box::new(Add), Some(Box::new(Value(2.0))))])]
    #[test_case(r"1 2 3 4 S0s \+ S2-r 0l /" => vec![Value(2.5)]; "calculate the average using repeat and stack size")]
    #[test_case("2 4 > 2 4 ?" => vec![Value(4.0)]; "max()")]
    #[test_case("2 4 < 2 4 ?" => vec![Value(2.0)]; "min()")]
    #[test_case("2 2 =" => vec![Value(1.0)]; "equality")]
    #[test_case("2 4 =" => vec![Value(0.0)]; "inequality")]
    #[test_case("2 (two) s c (two) l" => vec![Value(2.0)]; "storing a number in a named variable")]
    #[test_case(r"\- (minus) s (minus) l" => vec![Fn1(Box::new(Sub), None)]; "storing a function in a named variable")]
    #[test_case(r"\- (minus) s 2 1 (minus) $" => vec![Value(1.0)]; "applying a function from a named variable")]
    #[test_case(r"\? -1@ 1@ (positiveIfTrue)s 5 (positiveIfTrue)$" => vec![Value(1.0)]; "curried ternary operator")]
    #[test_case(r"\+ 1@ \* 2@ | (plus1Times2)s 4 (plus1Times2)$" => vec![Value(10.0)]; "composed functions")]
    #[test_case(r"\s 256@ \s257@ | \l257@ | \l256@ | \< | \l257@ | \l256@ | \? | (min)s 2 4 (min)$ 4 3 (min)$" => vec![Value(2.0), Value(3.0)]; "min() implementation")]
    fn evaluation(raw: &str) -> Vec<V> {
        let input = parse(raw).expect("parsing failed").1;
        dbg!(raw, &input);
        let mut machine = Machine::new();
        for v in input {
            dbg!(&v);
            machine.process(v).expect("processing failed");
            dbg!(&machine.stack);
        }
        machine.stack
    }
}
