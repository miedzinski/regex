use std::io::{self, Read};

use regex::ast::re;
use regex::dot::GraphvizCompiler;

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let ast = re(&input.trim()).unwrap().1;
    let mut visitor = GraphvizCompiler::new(io::stdout());
    visitor.render(&ast).unwrap();
}
