use nom::{
    branch::alt, character::complete::anychar, combinator::map, multi::many0,
    number::complete::double, IResult,
};
use std::io::stdin;

fn main() {
    for line in stdin().lines().filter_map(|l| l.ok()) {
        if line == "q" {
            break;
        }
        println!("{:?}", parse(&line));
    }
}

#[derive(Debug)]
enum V {
    Value(f64),
    Add,
    Sub,
    Mul,
    Div,
    Print,
}

#[derive(Debug, PartialEq)]
enum Item {
    Float(f64),
    Char(char),
}

fn parse(input: &str) -> IResult<&str, Vec<V>> {
    many0(alt((
        float,
        map(anychar, |c| match c {
            '+' => V::Add,
            '-' => V::Sub,
            '*' => V::Mul,
            '/' => V::Div,
            _ => unimplemented!(),
        }),
    )))(input)
}

fn float(input: &str) -> IResult<&str, V> {
    map(double, V::Value)(input)
}
