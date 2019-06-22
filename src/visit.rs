use crate::ast::*;

pub trait Visitor<T> {
    fn visit(&mut self, node: &Ast) -> T;
    fn visit_literal(&mut self, node: &Literal) -> T;
    fn visit_wildcard(&mut self, node: &Wildcard) -> T;
    fn visit_bracket(&mut self, node: &Bracket) -> T;
    fn visit_concatenation(&mut self, node: &Concatenation) -> T;
    fn visit_alternative(&mut self, node: &Alternative) -> T;
    fn visit_group(&mut self, node: &Group) -> T;
    fn visit_repetition(&mut self, node: &Repetition) -> T;
}

pub trait Visitable {
    fn accept<T>(&self, v: &mut Visitor<T>) -> T;
}

impl Visitable for Ast {
    fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        match self {
            Ast::Literal(x) => x.accept(v),
            Ast::Wildcard(x) => x.accept(v),
            Ast::Bracket(x) => x.accept(v),
            Ast::Concatenation(x) => x.accept(v),
            Ast::Alternative(x) => x.accept(v),
            Ast::Group(x) => x.accept(v),
            Ast::Repetition(x) => x.accept(v),
        }
    }
}

impl Visitable for Literal {
    fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        v.visit_literal(self)
    }
}

impl Visitable for Wildcard {
    fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        v.visit_wildcard(self)
    }
}

impl Visitable for Bracket {
    fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        v.visit_bracket(self)
    }
}

impl Visitable for Concatenation {
    fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        v.visit_concatenation(self)
    }
}

impl Visitable for Alternative {
    fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        v.visit_alternative(self)
    }
}

impl Visitable for Group {
    fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        v.visit_group(self)
    }
}

impl Visitable for Repetition {
    fn accept<T>(&self, v: &mut Visitor<T>) -> T {
        v.visit_repetition(self)
    }
}
