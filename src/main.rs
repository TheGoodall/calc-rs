use nom::{
    branch::alt, bytes::complete::tag, character::complete as cc, combinator::map,
    combinator::value, multi::many0, sequence::delimited, IResult,
};

use std::fmt;
use std::io::{stdin, stdout, Write};
use std::process::exit;
fn main() -> anyhow::Result<()> {
    let stdin = stdin();
    let mut stdout = stdout();

    let mut buffer = String::new();

    let mut line: Line = Line::new();

    loop {
        print!("> ");
        stdout.flush().unwrap();
        stdin.read_line(&mut buffer)?;

        if buffer == "exit\n" {
            exit(0)
        }

        match line.parse_add(&buffer) {
            Ok(()) => (),
            Err(ParseError::GenericParseError) => println!("Parsing Error!"),
        }
        buffer.clear();
        let calc_result = line.calc();
        match calc_result {
            Ok(stack) => {
                let answer = stack.last();

                match answer {
                    None => {}
                    Some(a) => {
                        println!("Stack: {stack}, Result: {a}")
                    }
                }
            }
            Err(CalcError::NotEnoughItemsInStack) => println!("Not enough items in stack"),
            Err(CalcError::MathError) => println!("Math Error!"),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Item {
    Num(i32),
    Operator(Operator),
}
impl Item {
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            map(cc::i32, Item::Num),
            map(Operator::parse, Item::Operator),
        ))(i)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Operator {
    Add,
    Multiply,
    Subtract,
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
        ))(i)
    }
}

#[derive(Debug, PartialEq)]
struct Line(Vec<Item>);

impl Line {
    fn new() -> Self {
        Line(vec![])
    }
    fn parse(i: &str) -> IResult<&str, Self> {
        map(
            many0(delimited(cc::multispace0, Item::parse, cc::multispace0)),
            Line,
        )(i)
    }

    fn parse_add(&mut self, i: &str) -> Result<(), ParseError> {
        let mut newline = Line::parse(i)
            .map_err(|_| (ParseError::GenericParseError))?
            .1;
        self.0.append(&mut newline.0);
        Ok(())
    }

    fn calc(&self) -> Result<Stack, CalcError> {
        let result = self.0.iter().fold(Ok(Stack::new()), |stack, item| {
            let mut stack = stack?;
            match item {
                Item::Num(number) => stack.0.push(*number),
                Item::Operator(op) => match op {
                    Operator::Add => {
                        let a = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        let b = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        stack.0.push(b + a)
                    }
                    Operator::Multiply => {
                        let a = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        let b = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        stack.0.push(b * a)
                    }
                    Operator::Subtract => {
                        let a = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        let b = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        stack.0.push(b - a)
                    }
                    Operator::Sum => {
                        let s = stack.0.iter().sum();
                        stack = Stack(vec![s])
                    }
                    Operator::Power => {
                        let a = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        let b = stack.0.pop().ok_or(CalcError::NotEnoughItemsInStack)?;
                        stack
                            .0
                            .push(b.pow(a.try_into().map_err(|_| CalcError::MathError)?))
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
#[derive(Debug, PartialEq)]
enum ParseError {
    GenericParseError,
}

#[derive(Debug, PartialEq)]
struct Stack(Vec<i32>);

impl Stack {
    fn new() -> Self {
        Stack(Vec::new())
    }
    fn last(&self) -> Option<&i32> {
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
        assert_eq!(Item::parse("32"), Ok(("", Item::Num(32))));
        assert_eq!(Item::parse("-73"), Ok(("", Item::Num(-73))));
    }
    #[test]
    fn test_operator_parsing() {
        assert_eq!(Item::parse("+"), Ok(("", Item::Operator(Operator::Add))));
        assert_eq!(
            Item::parse("*"),
            Ok(("", Item::Operator(Operator::Multiply)))
        );
    }
    #[test]
    fn test_single_item_parsing() {
        assert_eq!(Line::parse("3"), Ok(("", Line(vec![Item::Num(3)]))));
        assert_eq!(
            Line::parse("+"),
            Ok(("", Line(vec![Item::Operator(Operator::Add)])))
        );
        assert_eq!(
            Line::parse("*"),
            Ok(("", Line(vec![Item::Operator(Operator::Multiply)])))
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
                    Item::Num(3),
                    Item::Num(6),
                    Item::Operator(Operator::Add)
                ])
            ))
        );
        assert_eq!(
            Line::parse("47 9 + 15 *"),
            Ok((
                "",
                Line(vec![
                    Item::Num(47),
                    Item::Num(9),
                    Item::Operator(Operator::Add),
                    Item::Num(15),
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
                    Item::Num(3),
                    Item::Num(6),
                    Item::Num(-2),
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
                    Item::Num(3),
                    Item::Num(6),
                    Item::Operator(Operator::Multiply),
                    Item::Num(2),
                    Item::Operator(Operator::Multiply)
                ])
            ))
        );
        assert_eq!(
            Line::parse(" 3 6+2** "),
            Ok((
                "",
                Line(vec![
                    Item::Num(3),
                    Item::Num(6),
                    Item::Num(2),
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
                    Item::Num(3),
                    Item::Num(6),
                    Item::Num(2),
                    Item::Operator(Operator::Sum),
                ])
            ))
        );
        assert_eq!(
            Line::parse("3 2 ^"),
            Ok((
                "",
                Line(vec![
                    Item::Num(3),
                    Item::Num(2),
                    Item::Operator(Operator::Power),
                ])
            ))
        );
    }
    #[test]
    fn test_calculating() {
        assert_eq!(
            Line::parse("3 6 +").unwrap().1.calc().unwrap(),
            Stack(vec![9])
        );
        assert_eq!(
            Line::parse("3 6 *").unwrap().1.calc().unwrap(),
            Stack(vec![18])
        );
        assert_eq!(
            Line::parse("3 6 + 2 *").unwrap().1.calc().unwrap(),
            Stack(vec![18])
        );
        assert_eq!(
            Line::parse("6 3 - 2 -").unwrap().1.calc().unwrap(),
            Stack(vec![1])
        );
        assert_eq!(
            Line::parse("6 -3 - -2 -").unwrap().1.calc().unwrap(),
            Stack(vec![11])
        );
        assert_eq!(
            Line::parse("6 +3 - -2 *").unwrap().1.calc().unwrap(),
            Stack(vec![-6])
        );
        assert_eq!(
            Line::parse("2 4 6 S").unwrap().1.calc().unwrap(),
            Stack(vec![12])
        );
        assert_eq!(
            Line::parse("3 2 ^").unwrap().1.calc().unwrap(),
            Stack(vec![9])
        );
    }
    #[test]
    fn test_op_on_empty_stack() {
        assert_eq!(
            Line::parse("+").unwrap().1.calc(),
            Err(CalcError::NotEnoughItemsInStack)
        )
    }
}
