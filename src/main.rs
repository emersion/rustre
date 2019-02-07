extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;
mod parser;
mod rustfmt;

use std::io::{Read, stdout, stdin};
use crate::parser::*;
use crate::rustfmt::*;

fn main() {
	let mut buffer = String::new();
	stdin().read_to_string(&mut buffer).unwrap();

	let f = parse(&buffer).unwrap();
	eprintln!("{:?}", &f);

	&f.write_to(&mut stdout());
}
