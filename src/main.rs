use std::io::prelude::*;
use std::fs::File;
use std::rc::Rc;
use std::boxed::Box;

use ast::*;
use printer::Printable;

mod ast;
mod parser;
mod lexer;
mod printer;


fn main() {
    let mut f = File::open("cool.c").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s);
    let mut chars = s.chars().peekable();

    let tokens = lexer::lex(chars);
    let program = parser::parse_cool_program(&tokens);
    if let Some(p) = program {
        p.pretty_print(0);
    }
}
