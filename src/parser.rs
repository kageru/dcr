use nom::{
    branch::alt,
    character::complete::{anychar, char, multispace0},
    combinator::{map, map_res},
    multi::many0,
    number::complete::double,
    sequence::preceded,
    IResult, Parser,
};

use crate::V;

pub fn parse(input: &str) -> IResult<&str, Vec<V>> {
    many0(alt((add, float, op))).parse(input)
}

// Special case with higher precedence than float
// because the float parser allows a leading `+` which we don’t want.
fn add(input: &str) -> IResult<&str, V> {
    map(char('+'), |_| V::Add).parse(input)
}

fn op(input: &str) -> IResult<&str, V> {
    map_res(preceded(multispace0, anychar), |c| {
        Ok::<V, String>(match c {
            '+' => V::Add,
            '-' => V::Sub,
            '*' => V::Mul,
            '/' => V::Div,
            'p' => V::Print,
            'f' => V::Printall,
            'c' => V::Clear,
            'q' => V::Quit,
            _ => Err(format!("Unexpected input char ‘{c}’"))?,
        })
    })
    .parse(input)
}

fn float(input: &str) -> IResult<&str, V> {
    map(preceded(multispace0, double), V::Value).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::V::*;

    #[test]
    fn parse_test() {
        assert_eq!(
            parse("1 2+3-/*pf").expect("parsing failed").1[..],
            [
                Value(1.0),
                Value(2.0),
                Add,
                Value(3.0),
                Sub,
                Div,
                Mul,
                Print,
                Printall
            ],
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
}
