use std::io::{self, Write};

use crate::ast;

use crate::visit::{Visitable, Visitor};

pub struct GraphvizCompiler<W> {
    last: usize,
    output: W,
}

impl<W: Write> GraphvizCompiler<W> {
    pub fn new(output: W) -> GraphvizCompiler<W> {
        GraphvizCompiler { last: 0, output }
    }

    pub fn render(&mut self, ast: &ast::Ast) -> io::Result<()> {
        writeln!(self.output, "digraph {{\nrankdir = LR;")?;
        ast.accept(self)?;
        for node in 0..self.last {
            writeln!(self.output, "{} [shape = circle];", node)?;
        }
        writeln!(self.output, "{} [shape = doublecircle];\n}}", self.last)?;
        Ok(())
    }
}

impl<W: Write> Visitor<io::Result<()>> for GraphvizCompiler<W> {
    fn visit(&mut self, node: &ast::Ast) -> io::Result<()> {
        node.accept(self)
    }

    fn visit_literal(&mut self, node: &ast::Literal) -> io::Result<()> {
        self.last += 1;
        writeln!(
            self.output,
            "{} -> {} [label = {}];",
            self.last - 1,
            self.last,
            node.value()
        )
    }

    fn visit_wildcard(&mut self, _: &ast::Wildcard) -> io::Result<()> {
        self.last += 1;
        writeln!(
            self.output,
            "{} -> {} [label = ANY];",
            self.last - 1,
            self.last,
        )
    }

    fn visit_bracket(&mut self, node: &ast::Bracket) -> io::Result<()> {
        let start = self.last;
        let negated = if node.negated() { "not " } else { "" };
        for expr in node.exprs() {
            writeln!(self.output, "{} -> {} [label = ε];", start, self.last + 1)?;
            match expr {
                ast::BracketExpr::Char(c) => {
                    writeln!(
                        self.output,
                        "{} -> {} [label = \"{}{}\"];",
                        self.last + 1,
                        self.last + 2,
                        negated,
                        c
                    )?;
                }
                ast::BracketExpr::Range(a, b) => {
                    writeln!(
                        self.output,
                        "{} -> {} [label = \"{}{}-{}\"]",
                        self.last + 1,
                        self.last + 2,
                        negated,
                        a,
                        b
                    )?;
                }
                ast::BracketExpr::Class(class) => {
                    use ast::Class::*;
                    let trans = match class {
                        Alnum => "alphanumeric",
                        Alpha => "alpha",
                        Blank => "blank",
                        Cntrl => "control",
                        Digit => "digit",
                        Graph => "graph",
                        Lower => "lowercase",
                        Print => "printable",
                        Punct => "punctuation",
                        Space => "whitespace",
                        Upper => "uppercase",
                        Xdigit => "hexadecimal",
                    };
                    writeln!(
                        self.output,
                        "{} -> {} [label = \"{}{}\"];",
                        self.last + 1,
                        self.last + 2,
                        negated,
                        trans
                    )?;
                }
            }
            self.last += 2;
        }
        self.last += 1;
        for id in ((start + 2)..self.last).step_by(2) {
            writeln!(self.output, "{} -> {} [label = ε];", id, self.last)?;
        }
        Ok(())
    }

    fn visit_concatenation(&mut self, node: &ast::Concatenation) -> io::Result<()> {
        for node in node.items() {
            node.accept(self)?
        }
        Ok(())
    }

    fn visit_alternative(&mut self, node: &ast::Alternative) -> io::Result<()> {
        let start = self.last;
        let mut accepting = Vec::with_capacity(node.items().len());
        for node in node.items() {
            self.last += 1;
            writeln!(self.output, "{} -> {} [label = ε];", start, self.last)?;
            node.accept(self)?;
            accepting.push(self.last);
        }
        self.last += 1;
        for id in accepting {
            writeln!(self.output, "{} -> {} [label = ε];", id, self.last)?;
        }
        Ok(())
    }

    fn visit_group(&mut self, node: &ast::Group) -> io::Result<()> {
        node.inner().accept(self)
    }

    fn visit_repetition(&mut self, node: &ast::Repetition) -> io::Result<()> {
        use ast::Quantifier::*;
        match node.quantifier() {
            ZeroOrOne => {
                let start = self.last;
                node.inner().accept(self)?;
                writeln!(self.output, "{} -> {} [label = ε];", start, self.last)
            }
            ZeroOrMore => {
                let start = self.last;
                node.inner().accept(self)?;
                writeln!(self.output, "{} -> {} [label = ε];", start, self.last)?;
                writeln!(self.output, "{} -> {} [label = ε];", self.last, start)
            }
            OneOrMore => {
                node.inner().accept(self)?;
                let start = self.last;
                node.inner().accept(self)?;
                writeln!(self.output, "{} -> {} [label = ε];", start, self.last)?;
                writeln!(self.output, "{} -> {} [label = ε];", self.last, start)
            }
            Exact(n) => {
                for _ in 0..n {
                    node.inner().accept(self)?;
                }
                Ok(())
            }
            Minimum(n) => {
                for _ in 0..n {
                    node.inner().accept(self)?;
                }
                let start = self.last;
                node.inner().accept(self)?;
                writeln!(self.output, "{} -> {} [label = ε];", start, self.last)?;
                writeln!(self.output, "{} -> {} [label = ε];", self.last, start)
            }
            Range(n, m) => {
                let start = self.last;
                node.inner().accept(self)?;
                let len = self.last - start;
                for _ in 1..n {
                    node.inner().accept(self)?;
                }
                let end = self.last + ((m - n) as usize) * len;
                writeln!(self.output, "{} -> {} [label = ε];", self.last, end)?;
                for _ in 0..m - n {
                    node.inner().accept(self)?;
                    if end - self.last > 1 {
                        writeln!(self.output, "{} -> {} [label = ε];", self.last, end)?;
                    }
                }
                Ok(())
            }
        }
    }
}
