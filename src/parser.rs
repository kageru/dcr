use nom::{
    branch::alt,
    character::complete::{char, multispace0, one_of},
    combinator::{map, rest, value},
    multi::many0,
    number::complete::double,
    sequence::preceded,
    IResult, Parser,
};

use crate::V;

pub fn parse(input: &str) -> IResult<&str, Vec<V>> {
    many0(preceded(
        multispace0,
        alt((
            // These produce output
            map(alt((add, float, partial_op, op)), Some),
            // This is discarded
            comment,
        )),
    ))
    .map(|v| v.into_iter().flatten().collect())
    .parse(input)
}

fn comment(input: &str) -> IResult<&str, Option<V>> {
    value(None, preceded(char('#'), rest)).parse(input)
}

// Special case with higher precedence than float
// because the float parser allows a leading `+` which we don’t want.
fn add(input: &str) -> IResult<&str, V> {
    map(char('+'), |_| V::Add).parse(input)
}

const OP0: &str = "fcqS";
const OP1: &str = "p$";
const OP2: &str = "+-*/slr";
const OP3: &str = "?";

fn op(input: &str) -> IResult<&str, V> {
    alt((op0, op1, op2, op3)).parse(input)
}

fn partial_op(input: &str) -> IResult<&str, V> {
    preceded(
        char('\\'),
        alt((
            map(op1, |o| V::Fn(Box::new(o))),
            map(op2, |o| V::Fn1(Box::new(o), None)),
        )),
    )
    .parse(input)
}

fn op0(input: &str) -> IResult<&str, V> {
    map(one_of(OP0), |c| match c {
        'f' => V::Printall,
        'c' => V::Clear,
        'q' => V::Quit,
        'S' => V::Stacksize,
        _ => unreachable!(),
    })
    .parse(input)
}

fn op1(input: &str) -> IResult<&str, V> {
    map(one_of(OP1), |c| match c {
        'p' => V::Print,
        '$' => V::Apply,
        _ => unreachable!(),
    })
    .parse(input)
}

fn op2(input: &str) -> IResult<&str, V> {
    map(one_of(OP2), |c| match c {
        '+' => V::Add,
        '-' => V::Sub,
        '*' => V::Mul,
        '/' => V::Div,
        's' => V::Store,
        'l' => V::Load,
        'r' => V::Repeat,
        _ => unreachable!(),
    })
    .parse(input)
}

fn op3(input: &str) -> IResult<&str, V> {
    map(one_of(OP3), |c| match c {
        '?' => V::Conditional,
        _ => unreachable!(),
    })
    .parse(input)
}

fn float(input: &str) -> IResult<&str, V> {
    map(double, V::Value).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::V::*;

    fn assert_parses_as(s: &str, expected: &[V]) {
        let (rest, parsed) = parse(s).expect("parsing failed");
        assert_eq!("", rest);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_comment() {
        assert_parses_as("1 2+#gibberish", &[Value(1.0), Value(2.0), Add]);
        assert_parses_as(
            "1 2+      #--#+234     more  gibberish",
            &[Value(1.0), Value(2.0), Add],
        );
    }

    #[test]
    fn parse_expression() {
        assert_parses_as("1 2+3-", &[Value(1.0), Value(2.0), Add, Value(3.0), Sub]);
        assert_parses_as(
            "1 1-2--3",
            &[Value(1.0), Value(1.0), Value(-2.0), Sub, Value(-3.0)],
        );
        assert_parses_as(".5.5", &[Value(0.5), Value(0.5)]);
        assert_parses_as("4 4 +4", &[Value(4.0), Value(4.0), Add, Value(4.0)]);
        let operators = format!("{OP0}{OP1}{OP2}{OP3}");
        assert_parses_as(
            &operators,
            &[
                Printall,
                Clear,
                Quit,
                Stacksize,
                Print,
                Apply,
                Add,
                Sub,
                Mul,
                Div,
                Store,
                Load,
                Repeat,
                Conditional,
            ],
        );
    }

    #[test]
    fn partial_parsing() {
        assert_parses_as("\\++", &[Fn1(Box::new(Add), None), Add]);
    }
}
