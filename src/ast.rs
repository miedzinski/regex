use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, none_of, one_of},
    combinator::{map, opt},
    multi::{many1, separated_nonempty_list},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Ast {
    Literal(Literal),
    Wildcard(Wildcard),
    Bracket(Bracket),
    Concatenation(Concatenation),
    Alternative(Alternative),
    Group(Group),
    Repetition(Repetition),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Literal {
    value: char,
}

impl Literal {
    pub fn value(&self) -> char {
        self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wildcard;

#[derive(Clone, Debug, PartialEq)]
pub struct Bracket {
    exprs: Vec<BracketExpr>,
    negated: bool,
}

impl Bracket {
    pub fn exprs(&self) -> &[BracketExpr] {
        &self.exprs
    }

    pub fn negated(&self) -> bool {
        self.negated
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BracketExpr {
    Char(char),
    Range(char, char),
    Class(Class),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Class {
    Alnum,
    Alpha,
    Blank,
    Cntrl,
    Digit,
    Graph,
    Lower,
    Print,
    Punct,
    Space,
    Upper,
    Xdigit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Concatenation {
    items: Vec<Ast>,
}

impl Concatenation {
    pub fn items(&self) -> &[Ast] {
        &self.items
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Alternative {
    items: Vec<Ast>,
}

impl Alternative {
    pub fn items(&self) -> &[Ast] {
        &self.items
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Repetition {
    inner: Box<Ast>,
    quantifier: Quantifier,
}

impl Repetition {
    pub fn inner(&self) -> &Ast {
        &self.inner
    }

    pub fn quantifier(&self) -> Quantifier {
        self.quantifier
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Quantifier {
    /// ?
    ZeroOrOne,
    /// *
    ZeroOrMore,
    /// +
    OneOrMore,
    /// {n}
    Exact(u8),
    /// {n,}
    Minimum(u8),
    /// {n,m}
    Range(u8, u8),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    inner: Box<Ast>,
}

impl Group {
    pub fn inner(&self) -> &Ast {
        &self.inner
    }
}

fn number(i: &str) -> IResult<&str, u8> {
    map(digit1, |s| u8::from_str(s).unwrap())(i)
}

fn range(i: &str) -> IResult<&str, Quantifier> {
    alt((
        map(separated_pair(number, char(','), number), |(n, m)| {
            Quantifier::Range(n, m)
        }),
        map(terminated(number, char(',')), Quantifier::Minimum),
        map(number, Quantifier::Exact),
    ))(i)
}

fn quantifier(i: &str) -> IResult<&str, Quantifier> {
    alt((
        map(char('?'), |_| Quantifier::ZeroOrOne),
        map(char('*'), |_| Quantifier::ZeroOrMore),
        map(char('+'), |_| Quantifier::OneOrMore),
        delimited(char('{'), range, char('}')),
    ))(i)
}

fn group(i: &str) -> IResult<&str, Ast> {
    map(delimited(char('('), re, char(')')), |x| {
        Ast::Group(Group { inner: Box::new(x) })
    })(i)
}

fn escaped(i: &str) -> IResult<&str, char> {
    preceded(
        char('\\'),
        alt((
            one_of("\\\"'?|.+*()[]{}^$"),
            map(char('n'), |_| '\n'),
            map(char('r'), |_| '\r'),
            map(char('t'), |_| '\t'),
            map(char('a'), |_| '\x07'),
            map(char('e'), |_| '\x1b'),
            map(char('f'), |_| '\x0c'),
            map(char('v'), |_| '\x0b'),
            // XXX: unicode codepoints
        )),
    )(i)
}

fn literal(i: &str) -> IResult<&str, Ast> {
    map(alt((none_of("\\|.?+*(){}^$"), escaped)), |c| {
        Ast::Literal(Literal { value: c })
    })(i)
}

fn expr(i: &str) -> IResult<&str, Ast> {
    alt((
        bracket,
        literal,
        map(char('.'), |_| Ast::Wildcard(Wildcard)),
    ))(i)
}

fn basic_re(i: &str) -> IResult<&str, Ast> {
    alt((group, expr))(i)
}

fn simple_re(i: &str) -> IResult<&str, Ast> {
    let (i, ast) = basic_re(i)?;
    let (i, q) = opt(quantifier)(i)?;
    let ret = match q {
        Some(q) => Ast::Repetition(Repetition {
            inner: Box::new(ast),
            quantifier: q,
        }),
        None => ast,
    };
    Ok((i, ret))
}

fn branch(i: &str) -> IResult<&str, Ast> {
    let (i, v) = many1(simple_re)(i)?;
    let ret = if v.len() == 1 {
        v[0].clone()
    } else {
        Ast::Concatenation(Concatenation { items: v })
    };
    Ok((i, ret))
}

pub fn re(i: &str) -> IResult<&str, Ast> {
    let (i, v) = separated_nonempty_list(char('|'), branch)(i)?;
    let ret = if v.len() == 1 {
        v[0].clone()
    } else {
        Ast::Alternative(Alternative { items: v })
    };
    Ok((i, ret))
}

fn class_name(i: &str) -> IResult<&str, Class> {
    use Class::*;
    alt((
        map(tag("alnum"), |_| Alnum),
        map(tag("alpha"), |_| Alpha),
        map(tag("blank"), |_| Blank),
        map(tag("cntrl"), |_| Cntrl),
        map(tag("digit"), |_| Digit),
        map(tag("graph"), |_| Graph),
        map(tag("lower"), |_| Lower),
        map(tag("print"), |_| Print),
        map(tag("punct"), |_| Punct),
        map(tag("space"), |_| Space),
        map(tag("upper"), |_| Upper),
        map(tag("xdigit"), |_| Xdigit),
    ))(i)
}

fn class(i: &str) -> IResult<&str, Class> {
    delimited(tag("[:"), class_name, tag(":]"))(i)
}

fn bracket_literal(i: &str) -> IResult<&str, char> {
    alt((none_of(r"\]-"), escaped))(i)
}

fn range_expr(i: &str) -> IResult<&str, (char, char)> {
    separated_pair(bracket_literal, char('-'), bracket_literal)(i)
}

fn term(i: &str) -> IResult<&str, BracketExpr> {
    alt((
        map(range_expr, |(a, b)| BracketExpr::Range(a, b)),
        map(class, BracketExpr::Class),
        map(bracket_literal, BracketExpr::Char),
    ))(i)
}

fn bracket(i: &str) -> IResult<&str, Ast> {
    map(
        delimited(
            char('['),
            tuple((
                opt(char('^')),
                opt(one_of("]-")),
                many1(term),
                opt(char('-')),
            )),
            char(']'),
        ),
        |(negation, head, mut list, tail)| {
            let negated = negation.is_some();
            if let Some(head) = head {
                list.insert(0, BracketExpr::Char(head));
            }
            if let Some(tail) = tail {
                list.push(BracketExpr::Char(tail));
            }
            Ast::Bracket(Bracket {
                exprs: list,
                negated,
            })
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number() {
        assert_eq!(number("123"), Ok(("", 123)));
    }

    #[test]
    fn parse_range() {
        assert_eq!(range("2"), Ok(("", Quantifier::Exact(2))));
        assert_eq!(range("2,"), Ok(("", Quantifier::Minimum(2))));
        assert_eq!(range("2,3"), Ok(("", Quantifier::Range(2, 3))));
    }

    #[test]
    fn parse_quantifier() {
        assert_eq!(quantifier("?"), Ok(("", Quantifier::ZeroOrOne)));
        assert_eq!(quantifier("*"), Ok(("", Quantifier::ZeroOrMore)));
        assert_eq!(quantifier("+"), Ok(("", Quantifier::OneOrMore)));
        assert_eq!(quantifier("{2}"), Ok(("", Quantifier::Exact(2))));
        assert_eq!(quantifier("{2,}"), Ok(("", Quantifier::Minimum(2))));
        assert_eq!(quantifier("{2,3}"), Ok(("", Quantifier::Range(2, 3))));
    }

    #[test]
    fn parse_group() {
        assert!(group("()").is_err());
        assert_eq!(
            group("(foo)"),
            Ok((
                "",
                Ast::Group(Group {
                    inner: Box::new(Ast::Concatenation(Concatenation {
                        items: vec![
                            Ast::Literal(Literal { value: 'f' }),
                            Ast::Literal(Literal { value: 'o' }),
                            Ast::Literal(Literal { value: 'o' }),
                        ]
                    }))
                })
            ))
        );
        assert_eq!(
            group("((x))"),
            Ok((
                "",
                Ast::Group(Group {
                    inner: Box::new(Ast::Group(Group {
                        inner: Box::new(Ast::Literal(Literal { value: 'x' })),
                    })),
                })
            ))
        );
    }

    #[test]
    fn parse_literal() {
        assert_eq!(
            literal("abc"),
            Ok(("bc", Ast::Literal(Literal { value: 'a' })))
        );
        assert_eq!(
            literal(r"\ab"),
            Ok(("b", Ast::Literal(Literal { value: '\x07' })))
        );
        assert!(literal("\\").is_err());
        assert!(literal(".").is_err());
        assert_eq!(
            literal(" x"),
            Ok(("x", Ast::Literal(Literal { value: ' ' })))
        );
    }

    #[test]
    fn parse_expr() {
        assert_eq!(
            expr("foo"),
            Ok(("oo", Ast::Literal(Literal { value: 'f' })))
        );
        assert_eq!(expr(".x"), Ok(("x", Ast::Wildcard(Wildcard))));
    }

    #[test]
    fn parse_basic_re() {
        assert_eq!(
            basic_re("(f)oo"),
            Ok((
                "oo",
                Ast::Group(Group {
                    inner: Box::new(Ast::Literal(Literal { value: 'f' }))
                })
            ))
        );
        assert_eq!(basic_re(".oof"), Ok(("oof", Ast::Wildcard(Wildcard))));
    }

    #[test]
    fn parse_simple_re() {
        assert_eq!(
            simple_re("foo"),
            Ok(("oo", Ast::Literal(Literal { value: 'f' })))
        );
        assert_eq!(
            simple_re("(ab)c"),
            Ok((
                "c",
                Ast::Group(Group {
                    inner: Box::new(Ast::Concatenation(Concatenation {
                        items: vec![
                            Ast::Literal(Literal { value: 'a' }),
                            Ast::Literal(Literal { value: 'b' }),
                        ]
                    }))
                })
            ))
        );
        assert_eq!(
            simple_re(".+."),
            Ok((
                ".",
                Ast::Repetition(Repetition {
                    inner: Box::new(Ast::Wildcard(Wildcard)),
                    quantifier: Quantifier::OneOrMore,
                })
            ))
        );
    }

    #[test]
    fn parse_branch() {
        assert_eq!(
            branch("foo"),
            Ok((
                "",
                Ast::Concatenation(Concatenation {
                    items: vec![
                        Ast::Literal(Literal { value: 'f' }),
                        Ast::Literal(Literal { value: 'o' }),
                        Ast::Literal(Literal { value: 'o' }),
                    ]
                })
            ))
        );
        assert_eq!(
            branch("a.?b"),
            Ok((
                "",
                Ast::Concatenation(Concatenation {
                    items: vec![
                        Ast::Literal(Literal { value: 'a' }),
                        Ast::Repetition(Repetition {
                            inner: Box::new(Ast::Wildcard(Wildcard)),
                            quantifier: Quantifier::ZeroOrOne,
                        }),
                        Ast::Literal(Literal { value: 'b' }),
                    ]
                })
            ))
        );
    }

    #[test]
    fn parse_re() {
        assert_eq!(re("a"), Ok(("", Ast::Literal(Literal { value: 'a' }))));
        assert_eq!(
            re("a|b|c"),
            Ok((
                "",
                Ast::Alternative(Alternative {
                    items: vec![
                        Ast::Literal(Literal { value: 'a' }),
                        Ast::Literal(Literal { value: 'b' }),
                        Ast::Literal(Literal { value: 'c' }),
                    ]
                })
            ))
        );
        assert_eq!(
            re("a{2}|.(b)"),
            Ok((
                "",
                Ast::Alternative(Alternative {
                    items: vec![
                        Ast::Repetition(Repetition {
                            inner: Box::new(Ast::Literal(Literal { value: 'a' })),
                            quantifier: Quantifier::Exact(2),
                        }),
                        Ast::Concatenation(Concatenation {
                            items: vec![
                                Ast::Wildcard(Wildcard),
                                Ast::Group(Group {
                                    inner: Box::new(Ast::Literal(Literal { value: 'b' }))
                                }),
                            ]
                        }),
                    ]
                })
            ))
        );
    }

    #[test]
    fn parse_class_name() {
        assert_eq!(class_name("alnum"), Ok(("", Class::Alnum)));
        assert!(class_name("foo").is_err());
    }

    #[test]
    fn parse_class() {
        assert_eq!(class("[:alpha:]"), Ok(("", Class::Alpha)));
        assert!(class("[::]").is_err());
    }

    #[test]
    fn parse_bracket_litera() {
        assert_eq!(bracket_literal("abc"), Ok(("bc", 'a')));
        assert!(bracket_literal("\\").is_err());
        assert_eq!(bracket_literal("."), Ok(("", '.')));
    }

    #[test]
    fn parse_range_expr() {
        assert_eq!(range_expr("a-bc"), Ok(("c", ('a', 'b'))));
    }

    #[test]
    fn parse_term() {
        assert_eq!(term("a-bc"), Ok(("c", BracketExpr::Range('a', 'b'))));
        assert_eq!(
            term("[:space:]"),
            Ok(("", BracketExpr::Class(Class::Space))),
        );
        assert_eq!(term("foo"), Ok(("oo", BracketExpr::Char('f'))));
    }

    #[test]
    fn parse_bracket() {
        assert!(bracket("[]").is_err());
        assert_eq!(
            bracket("[a]"),
            Ok((
                "",
                Ast::Bracket(Bracket {
                    exprs: vec![BracketExpr::Char('a')],
                    negated: false,
                }),
            )),
        );
        assert_eq!(
            bracket("[[:digit:]]"),
            Ok((
                "",
                Ast::Bracket(Bracket {
                    exprs: vec![BracketExpr::Class(Class::Digit)],
                    negated: false,
                }),
            )),
        );
        assert_eq!(
            bracket("[a-z]"),
            Ok((
                "",
                Ast::Bracket(Bracket {
                    exprs: vec![BracketExpr::Range('a', 'z')],
                    negated: false,
                }),
            )),
        );
        assert_eq!(
            bracket("[^abc]"),
            Ok((
                "",
                Ast::Bracket(Bracket {
                    exprs: vec![
                        BracketExpr::Char('a'),
                        BracketExpr::Char('b'),
                        BracketExpr::Char('c'),
                    ],
                    negated: true,
                }),
            )),
        );
        assert_eq!(
            bracket("[^]a-]"),
            Ok((
                "",
                Ast::Bracket(Bracket {
                    exprs: vec![
                        BracketExpr::Char(']'),
                        BracketExpr::Char('a'),
                        BracketExpr::Char('-'),
                    ],
                    negated: true,
                }),
            )),
        );
    }
}
