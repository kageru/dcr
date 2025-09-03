use nom::{
    branch::alt,
    character::complete::{anychar, char, multispace0},
    combinator::{all_consuming, map},
    multi::many0,
    number::complete::double,
    sequence::preceded,
    IResult, Parser,
};
use std::{io::stdin, ops};
use V::*;

fn main() {
    let mut stack = Vec::new();
    for line in stdin().lines().filter_map(|l| l.ok()) {
        for v in parse(&line).unwrap().1 {
            if let Err(e) = process(&mut stack, v) {
                eprintln!("Error at {v:?}: {e} (stack was {stack:?})")
            }
        }
    }
}

const STACK_EMPTY: &str = "not enough elements on the stack";

fn process(stack: &mut Vec<f64>, v: V) -> Result<(), &'static str> {
    Ok(match v {
        Value(v) => stack.push(v),
        Add => binop(stack, ops::Add::add)?,
        Sub => binop(stack, ops::Sub::sub)?,
        Mul => binop(stack, ops::Mul::mul)?,
        Div => binop(stack, ops::Div::div)?,
        Print => println!("{}", stack.last().ok_or(STACK_EMPTY)?),
        Printall => println!("{stack:?}"),
        Clear => stack.clear(),
        Quit => std::process::exit(0),
    })
}

fn binop<F: FnOnce(f64, f64) -> f64>(stack: &mut Vec<f64>, f: F) -> Result<(), &'static str> {
    let (a, b) = pop2(stack)?;
    Ok(stack.push(f(a, b)))
}

fn pop2(v: &mut Vec<f64>) -> Result<(f64, f64), &'static str> {
    // Checking first rather than `pop()?` because we don’t want to pop at all if there aren’t enough values.
    let (&a, &b) = match v.as_slice() {
        [.., a, b] => (a, b),
        _ => return Err(STACK_EMPTY),
    };
    v.pop();
    v.pop();
    Ok((a, b))
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum V {
    Value(f64),
    Add,
    Sub,
    Mul,
    Div,
    Print,
    Quit,
    Clear,
    Printall,
}

fn parse(input: &str) -> IResult<&str, Vec<V>> {
    all_consuming(many0(alt((add, float, op)))).parse(input)
}

// Special case with higher precedence than float
// because the float parser allows a leading `+` which we don’t want.
fn add(input: &str) -> IResult<&str, V> {
    map(char('+'), |_| V::Add).parse(input)
}

fn op(input: &str) -> IResult<&str, V> {
    map(preceded(multispace0, anychar), |c| match c {
        '+' => V::Add,
        '-' => V::Sub,
        '*' => V::Mul,
        '/' => V::Div,
        'p' => V::Print,
        'f' => V::Printall,
        'c' => V::Clear,
        'q' => V::Quit,
        _ => unimplemented!(),
    })
    .parse(input)
}

fn float(input: &str) -> IResult<&str, V> {
    map(preceded(multispace0, double), V::Value).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        assert_eq!(
            parse("1 2+3-").expect("parsing failed").1[..],
            [Value(1.0), Value(2.0), Add, Value(3.0), Sub],
        );
        assert_eq!(
            parse("1 1-2--3").expect("parsing failed").1[..],
            [Value(1.0), Value(1.0), Value(-2.0), Sub, Value(-3.0)],
        );
        assert_eq!(
            parse(".5.5").expect("parsing failed").1[..],
            [Value(0.5), Value(0.5)],
        );
    }

    #[test]
    fn evaluation_test() {
        for (input, expectation) in [
            ("1 2+3-", vec![0.0]),
            ("40 2+6/7*", vec![49.0]),
            ("5 2/3+3", vec![5.5, 3.0]),
        ] {
            let input = parse(input).expect("parsing failed").1;
            let mut stack = Vec::new();
            for v in input {
                process(&mut stack, v).expect("processing failed");
            }
            assert_eq!(stack, expectation);
        }
    }
}
