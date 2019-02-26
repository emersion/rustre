extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;
mod nast;
mod normalizer;
mod parser;
mod rustfmt;
mod typer;

use std::io::{Read, stdout, stdin};
use crate::normalizer::normalize;
use crate::parser::parse;
use crate::rustfmt::WriterTo;

fn main() {
	let mut buffer = String::new();
	stdin().read_to_string(&mut buffer).unwrap();

	let f = parse(&buffer).unwrap();
	eprintln!("{:?}", &f);

	let nf = normalize(&f);
	eprintln!("{:?}", &nf);

	&nf.write_to(&mut stdout()).unwrap();
}
