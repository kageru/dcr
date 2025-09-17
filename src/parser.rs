use crate::V;
use nom::{
    branch::alt,
    character::complete::{alphanumeric1, char, digit0, digit1, multispace0, one_of},
    combinator::{map, opt, recognize, rest, value, verify},
    multi::many0,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub fn parse(input: &str) -> IResult<&str, Vec<V>> {
    let function_mode = AtomicBool::new(false);
    let functions = AtomicUsize::new(0);
    many0(preceded(
        multispace0,
        alt((
            // These produce output
            map(float, |f| {
                // While in function mode, automatically curry values.
                if function_mode.load(Ordering::Relaxed) {
                    vec![V::Value(f), V::Curry]
                } else {
                    vec![V::Value(f)]
                }
            }),
            map(
                alt((
                    partial_op,
                    // if we’re in function mode, all functions except the curry operator should be lazy
                    verify(op, |op| {
                        cannot_be_lazy(op) || !function_mode.load(Ordering::Relaxed)
                    }),
                    identifier,
                )),
                |o| vec![o],
            ),
            map(partial_op_inner, |o| {
                if functions.fetch_add(1, Ordering::Relaxed) <= 1 {
                    vec![o]
                } else {
                    vec![V::Compose, o]
                }
            }),
            map(char('}'), |_| {
                function_mode.store(false, Ordering::Relaxed);
                if functions.load(Ordering::Relaxed) >= 2 {
                    vec![V::Compose]
                } else {
                    Vec::new()
                }
            }),
            // This is discarded
            map(char('{'), |_| {
                function_mode.store(true, Ordering::Relaxed);
                functions.store(0, Ordering::Relaxed);
                Vec::new()
            }),
            value(Vec::new(), comment),
        )),
    ))
    .map(|v| v.into_iter().flatten().collect())
    .parse(input)
}

fn cannot_be_lazy(op: &V) -> bool {
    matches!(op, &V::Curry | &V::Compose)
}

fn identifier(input: &str) -> IResult<&str, V> {
    map(delimited(char('('), alphanumeric1, char(')')), |s: &str| {
        V::Identifier(s.to_owned())
    })
    .parse(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    preceded(char('#'), rest).parse(input)
}

const OP0: &str = "fcqS";
const OP1: &str = "p$";
const OP2: &str = "+-*/%slr<>=|@";
const OP3: &str = "?";

fn op(input: &str) -> IResult<&str, V> {
    alt((op0, op1, op2, op3)).parse(input)
}

fn partial_op(input: &str) -> IResult<&str, V> {
    preceded(char('\\'), partial_op_inner).parse(input)
}

fn partial_op_inner(input: &str) -> IResult<&str, V> {
    map(op, |o| V::Fun(Box::new(o))).parse(input)
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
        '%' => V::Mod,
        's' => V::Store,
        'l' => V::Load,
        'r' => V::Repeat,
        '<' => V::LessThan,
        '>' => V::GreaterThan,
        '=' => V::Equal,
        '|' => V::Compose,
        '@' => V::Curry,
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

fn float(input: &str) -> IResult<&str, f64> {
    map(
        recognize((
            opt(char('-')),
            // Parsers are greedy, so we need both cases
            alt((
                (digit0, opt(char('.')), digit1),
                (digit1, opt(char('.')), digit0),
            )),
        )),
        |s: &str| s.parse().expect(&format!("Failed to parse {s} as float")),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::V::*;
    use test_case::test_case;

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

    #[test_case("1" => 1.0)]
    #[test_case("1.0" => 1.0)]
    #[test_case("1." => 1.0; "trailing dot")]
    #[test_case("01.00" => 1.0)]
    #[test_case(".5" => 0.5; "leading dot")]
    #[test_case("0.5" => 0.5)]
    fn float_parser(s: &str) -> f64 {
        match float(s) {
            Ok(("", f)) => f,
            e => panic!("{e:?}"),
        }
    }

    #[test_case("asdf")]
    #[test_case("a1")]
    fn reject_invalid_floats(s: &str) {
        assert!(matches!(float(s), Err(_)));
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
                Mod,
                Store,
                Load,
                Repeat,
                LessThan,
                GreaterThan,
                Equal,
                Compose,
                Curry,
                Conditional,
            ],
        );
    }

    #[test]
    fn parse_identifiers() {
        assert_parses_as(
            "(asd)(sdf2)",
            &[Identifier("asd".to_owned()), Identifier("sdf2".to_owned())],
        );
    }

    #[test]
    fn partial_parsing() {
        assert_parses_as("\\++", &[Fun(Box::new(Add)), Add]);
        assert_parses_as("\\-", &[Fun(Box::new(Sub))]);
    }

    #[test]
    fn function_mode() {
        assert_parses_as("{*2", &[Fun(Box::new(Mul)), Value(2.0), Curry]);
        assert_parses_as("{+2}", &[Fun(Box::new(Add)), Value(2.0), Curry]);
        assert_parses_as(
            "{?+@-@}",
            &[
                Fun(Box::new(Conditional)),
                Fun(Box::new(Add)),
                Curry,
                Compose,
                Fun(Box::new(Sub)),
                Curry,
                Compose,
            ],
        );
    }
}
