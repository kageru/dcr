use nom::{
    branch::alt,
    character::complete::{char, multispace0, one_of},
    combinator::map,
    multi::many0,
    number::complete::double,
    sequence::preceded,
    IResult, Parser,
};

use crate::V;

pub fn parse(input: &str) -> IResult<&str, Vec<V>> {
    many0(preceded(multispace0, alt((add, float, op)))).parse(input)
}

// Special case with higher precedence than float
// because the float parser allows a leading `+` which we don’t want.
fn add(input: &str) -> IResult<&str, V> {
    map(char('+'), |_| V::Add).parse(input)
}

const OPERATORS: &str = "+-*/pfslcq";

fn op(input: &str) -> IResult<&str, V> {
    map(one_of(OPERATORS), |c| match c {
        '+' => V::Add,
        '-' => V::Sub,
        '*' => V::Mul,
        '/' => V::Div,
        'p' => V::Print,
        'f' => V::Printall,
        's' => V::Store,
        'l' => V::Load,
        'c' => V::Clear,
        'q' => V::Quit,
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
    fn parse_test() {
        assert_parses_as("1 2+3-", &[Value(1.0), Value(2.0), Add, Value(3.0), Sub]);
        assert_parses_as(
            "1 1-2--3",
            &[Value(1.0), Value(1.0), Value(-2.0), Sub, Value(-3.0)],
        );
        assert_parses_as(".5.5", &[Value(0.5), Value(0.5)]);
        assert_parses_as("4 4 +4", &[Value(4.0), Value(4.0), Add, Value(4.0)]);
        assert_parses_as(
            OPERATORS,
            &[
                Add, Sub, Mul, Div, Print, Printall, Store, Load, Clear, Quit,
            ],
        );
    }
}
