pub const STDLIB: &[&str] = &[MIN, MAX, REDUCE];

const MIN: &str = "{ s256 s257 l257 l256 < l257 l256 ? }(min)s";
const MAX: &str = "{ s256 s257 l257 l256 > l257 l256 ? }(max)s";
const REDUCE: &str = "{ S - 2 r }(reduce)s";
// const AVERAGE: &str = "{ S s256 } (reduce)l | { / l256 }";

#[cfg(test)]
mod tests {
    use crate::{machine::Machine, parser::parse, V};
    use test_case::test_case;

    fn expect_single_result(raw: &str) -> f64 {
        let input = parse(raw).expect("parsing failed").1;
        let mut machine = Machine::new();
        for v in input {
            machine.process(v).expect("processing failed");
        }
        match machine.stack.as_slice() {
            [V::Value(f)] => *f,
            s => panic!("stack should be a single value but was {s:?}"),
        }
    }

    #[test_case(2.0, 4.0 => 2.0)]
    #[test_case(4.0, 2.0 => 2.0)]
    #[test_case(4.0, 4.0 => 4.0)]
    #[test_case(2.0, -2.0 => -2.0)]
    #[test_case(0.0, 0.00001 => 0.0)]
    fn min(a: f64, b: f64) -> f64 {
        expect_single_result(&format!("{a} {b} (min)$"))
    }

    #[test_case(2.0, 4.0 => 4.0)]
    #[test_case(4.0, 2.0 => 4.0)]
    #[test_case(4.0, 4.0 => 4.0)]
    #[test_case(2.0, -2.0 => 2.0)]
    #[test_case(0.0, 0.00001 => 0.00001)]
    fn max(a: f64, b: f64) -> f64 {
        expect_single_result(&format!("{a} {b} (max)$"))
    }

    #[test]
    fn reduce() {
        assert_eq!(expect_single_result("1 2 3 4 5 6 7 8 \\+ (reduce)$"), 36.0);
    }
}
