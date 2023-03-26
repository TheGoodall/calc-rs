use nom::{
    branch::alt, bytes::complete::tag, character::complete as cc, combinator::map,
    combinator::value, multi::many0, sequence::delimited, IResult,
};

use num::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Rational64};
use std::fmt;
use std::io::{stdin, stdout, Write};
use std::process::exit;

fn main() -> anyhow::Result<()> {
    let stdin = stdin();
    let mut stdout = stdout();

    let mut buffer = String::new();
    let mut stack = Stack::new();

    loop {
        print!("> ");
        stdout.flush().unwrap();
        stdin.read_line(&mut buffer)?;

        if buffer == "exit\n" {
            exit(0)
        }

        match Line::parse(&buffer) {
            Err(_) => println!("Parsing Error!"),
            Ok((_, line)) => {
                buffer.clear();
                let calc_result = line.calc(stack.clone());
                if let Some(returned_stack) = match calc_result {
                    Ok(returned_stack) => {
                        match returned_stack.last() {
                            None => {}
                            Some(a) => {
                                println!("Stack: {returned_stack}, Result: {a}")
                            }
                        };
                        Some(returned_stack)
                    }
                    Err(CalcError::NotEnoughItemsInStack) => {
                        println!("Stack: {stack}, Not enough items in stack!");
                        None
                    }
                    Err(CalcError::MathError) => {
                        println!("Stack: {stack}, Math Error!");
                        None
                    }
                } {
                    stack = returned_stack
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Item {
    Num(Rational64),
    Operator(Operator),
}
impl Item {
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            map(cc::i64, |i| Item::Num(Rational64::from(i))),
            map(Operator::parse, Item::Operator),
        ))(i)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Operator {
    Add,
    Multiply,
    Subtract,
    Divide,
    Sum,
    Power,
    Clear,
}

impl Operator {
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            value(Operator::Add, tag("+")),
            value(Operator::Multiply, tag("*")),
            value(Operator::Subtract, tag("-")),
            value(Operator::Sum, tag("S")),
            value(Operator::Power, tag("^")),
            value(Operator::Clear, tag("c")),
            value(Operator::Divide, tag("/")),
        ))(i)
    }
}

#[derive(Debug, PartialEq, Default)]
struct Line(Vec<Item>);

impl Line {
    fn parse(i: &str) -> IResult<&str, Self> {
        map(
            many0(delimited(cc::multispace0, Item::parse, cc::multispace0)),
            Line,
        )(i)
    }

    fn calc(&self, existing_stack: Stack) -> Result<Stack, CalcError> {
        let result = self.0.iter().fold(Ok(existing_stack), |stack, item| {
            let mut stack = stack?;
            match item {
                Item::Num(number) => stack.0.push(*number),
                Item::Operator(op) => match op {
                    Operator::Add => {
                        let a = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        let b = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        stack.0.push(b.checked_add(&a).ok_or(CalcError::MathError)?)
                    }
                    Operator::Multiply => {
                        let a = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        let b = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        stack.0.push(b.checked_mul(&a).ok_or(CalcError::MathError)?)
                    }
                    Operator::Subtract => {
                        let a = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        let b = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        stack.0.push(b.checked_sub(&a).ok_or(CalcError::MathError)?)
                    }
                    Operator::Divide => {
                        let a = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        let b = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        stack.0.push(b.checked_div(&a).ok_or(CalcError::MathError)?)
                    }
                    Operator::Sum => {
                        let s = stack.0.iter().sum();
                        stack = Stack(vec![s])
                    }
                    Operator::Power => {
                        let a = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        let b = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        stack.0.push(
                            b.pow(
                                a.to_integer()
                                    .try_into()
                                    .map_err(|_| CalcError::MathError)?,
                            ),
                        )
                    }
                    Operator::Clear => stack = Stack(vec![]),
                },
            };
            Ok(stack)
        });

        result
    }
}

#[derive(Debug, PartialEq)]
enum CalcError {
    NotEnoughItemsInStack,
    MathError,
}

#[derive(Debug, PartialEq, Clone)]
struct Stack(Vec<Rational64>);

impl Stack {
    fn new() -> Self {
        Stack(Vec::new())
    }
    fn last(&self) -> Option<&Rational64> {
        self.0.last()
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().for_each(|i| write!(f, " {}", i).unwrap());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_number_parsing() {
        assert_eq!(
            Item::parse("32"),
            Ok(("", Item::Num(Rational64::from_integer(32))))
        );
        assert_eq!(
            Item::parse("-73"),
            Ok(("", Item::Num(Rational64::from_integer(-73))))
        );
    }
    #[test]
    fn test_operator_parsing() {
        assert_eq!(Item::parse("+"), Ok(("", Item::Operator(Operator::Add))));
        assert_eq!(Item::parse("/"), Ok(("", Item::Operator(Operator::Divide))));
        assert_eq!(
            Item::parse("-"),
            Ok(("", Item::Operator(Operator::Subtract)))
        );
        assert_eq!(
            Item::parse("*"),
            Ok(("", Item::Operator(Operator::Multiply)))
        );
    }
    #[test]
    fn test_single_item_parsing() {
        assert_eq!(
            Line::parse("3"),
            Ok(("", Line(vec![Item::Num(Rational64::from_integer(3))])))
        );
        assert_eq!(
            Line::parse("+"),
            Ok(("", Line(vec![Item::Operator(Operator::Add)])))
        );
        assert_eq!(
            Line::parse("-"),
            Ok(("", Line(vec![Item::Operator(Operator::Subtract)])))
        );
        assert_eq!(
            Line::parse("*"),
            Ok(("", Line(vec![Item::Operator(Operator::Multiply)])))
        );
        assert_eq!(
            Line::parse("/"),
            Ok(("", Line(vec![Item::Operator(Operator::Divide)])))
        );
        assert_eq!(
            Line::parse("S"),
            Ok(("", Line(vec![Item::Operator(Operator::Sum)])))
        );
        assert_eq!(
            Line::parse("-"),
            Ok(("", Line(vec![Item::Operator(Operator::Subtract)])))
        );
        assert_eq!(
            Line::parse("^"),
            Ok(("", Line(vec![Item::Operator(Operator::Power)])))
        );
    }
    #[test]
    fn test_multiple_item_parsing() {
        assert_eq!(
            Line::parse("3 6 +"),
            Ok((
                "",
                Line(vec![
                    Item::Num(Rational64::from_integer(3)),
                    Item::Num(Rational64::from_integer(6)),
                    Item::Operator(Operator::Add)
                ])
            ))
        );
        assert_eq!(
            Line::parse("47 9 + 15 *"),
            Ok((
                "",
                Line(vec![
                    Item::Num(Rational64::from_integer(47)),
                    Item::Num(Rational64::from_integer(9)),
                    Item::Operator(Operator::Add),
                    Item::Num(Rational64::from_integer(15)),
                    Item::Operator(Operator::Multiply)
                ])
            ))
        );
    }
    #[test]
    fn test_multiple_item_parsing_with_weird_spaces() {
        assert_eq!(
            Line::parse("3 6-2**"),
            Ok((
                "",
                Line(vec![
                    Item::Num(Rational64::from_integer(3)),
                    Item::Num(Rational64::from_integer(6)),
                    Item::Num(Rational64::from_integer(-2)),
                    Item::Operator(Operator::Multiply),
                    Item::Operator(Operator::Multiply)
                ])
            ))
        );
        assert_eq!(
            Line::parse(" 3 6*2* "),
            Ok((
                "",
                Line(vec![
                    Item::Num(Rational64::from_integer(3)),
                    Item::Num(Rational64::from_integer(6)),
                    Item::Operator(Operator::Multiply),
                    Item::Num(Rational64::from_integer(2)),
                    Item::Operator(Operator::Multiply)
                ])
            ))
        );
        assert_eq!(
            Line::parse(" 3 6+2** "),
            Ok((
                "",
                Line(vec![
                    Item::Num(Rational64::from_integer(3)),
                    Item::Num(Rational64::from_integer(6)),
                    Item::Num(Rational64::from_integer(2)),
                    Item::Operator(Operator::Multiply),
                    Item::Operator(Operator::Multiply)
                ])
            ))
        );
        assert_eq!(
            Line::parse("3 6 2 S"),
            Ok((
                "",
                Line(vec![
                    Item::Num(Rational64::from_integer(3)),
                    Item::Num(Rational64::from_integer(6)),
                    Item::Num(Rational64::from_integer(2)),
                    Item::Operator(Operator::Sum),
                ])
            ))
        );
        assert_eq!(
            Line::parse("3 2 ^"),
            Ok((
                "",
                Line(vec![
                    Item::Num(Rational64::from_integer(3)),
                    Item::Num(Rational64::from_integer(2)),
                    Item::Operator(Operator::Power),
                ])
            ))
        );
    }
    #[test]
    fn test_calculating_simple_integers() {
        assert_eq!(
            Line::parse("3 6 +").unwrap().1.calc(Stack::new()).unwrap(),
            Stack(vec![Rational64::from_integer(9)])
        );
        assert_eq!(
            Line::parse("3 6 *").unwrap().1.calc(Stack::new()).unwrap(),
            Stack(vec![Rational64::from_integer(18)])
        );
        assert_eq!(
            Line::parse("3 6 + 2 *")
                .unwrap()
                .1
                .calc(Stack::new())
                .unwrap(),
            Stack(vec![Rational64::from_integer(18)])
        );
        assert_eq!(
            Line::parse("6 3 - 2 -")
                .unwrap()
                .1
                .calc(Stack::new())
                .unwrap(),
            Stack(vec![Rational64::from_integer(1)])
        );
        assert_eq!(
            Line::parse("6 -3 - -2 -")
                .unwrap()
                .1
                .calc(Stack::new())
                .unwrap(),
            Stack(vec![Rational64::from_integer(11)])
        );
        assert_eq!(
            Line::parse("6 +3 - -2 *")
                .unwrap()
                .1
                .calc(Stack::new())
                .unwrap(),
            Stack(vec![Rational64::from_integer(-6)])
        );
        assert_eq!(
            Line::parse("2 4 6 S")
                .unwrap()
                .1
                .calc(Stack::new())
                .unwrap(),
            Stack(vec![Rational64::from_integer(12)])
        );
        assert_eq!(
            Line::parse("3 2 ^").unwrap().1.calc(Stack::new()).unwrap(),
            Stack(vec![Rational64::from_integer(9)])
        );
    }
    #[test]
    fn test_op_on_empty_stack() {
        assert_eq!(
            Line::parse("+").unwrap().1.calc(Stack::new()),
            Err(CalcError::NotEnoughItemsInStack)
        );
        assert_eq!(
            Line::parse("1+").unwrap().1.calc(Stack::new()),
            Err(CalcError::NotEnoughItemsInStack)
        )
    }

    #[test]
    fn test_creating_fraction() {
        assert_eq!(
            Line::parse("1 2 /").unwrap().1.calc(Stack::new()).unwrap(),
            Stack(vec![Rational64::new(1, 2)])
        );
        assert_eq!(
            Line::parse("2 4 /").unwrap().1.calc(Stack::new()).unwrap(),
            Stack(vec![Rational64::new(1, 2)])
        )
    }

    #[test]
    fn operations_on_fractions() {
        assert_eq!(
            Line::parse("1 2 / 1 2 / +")
                .unwrap()
                .1
                .calc(Stack::new())
                .unwrap(),
            Stack(vec![Rational64::from_integer(1)])
        );
        assert_eq!(
            Line::parse("1 2 / 1 2 / *")
                .unwrap()
                .1
                .calc(Stack::new())
                .unwrap(),
            Stack(vec![Rational64::new(1, 4)])
        );
        assert_eq!(
            Line::parse("1 2 / 1 2 / -")
                .unwrap()
                .1
                .calc(Stack::new())
                .unwrap(),
            Stack(vec![Rational64::from_integer(0)])
        );
    }
}
