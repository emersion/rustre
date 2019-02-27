extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;
mod nast;
mod normalizer;
mod parser;
mod rustfmt;
mod typer;
mod sequentializer;

use std::io::{Read, stdout, stdin};
use crate::parser::parse;
use crate::rustfmt::WriterTo;
use crate::normalizer::normalize;
use crate::sequentializer::sequentialize;

fn main() {
	let mut buffer = String::new();
	stdin().read_to_string(&mut buffer).unwrap();

	let f = parse(&buffer).unwrap();
	eprintln!("{:?}", &f);

	let nf = normalize(&f);
	eprintln!("{:?}", &nf);

	let sf = sequentialize(&nf);
	eprintln!("{:?}", &sf);

	&sf.write_to(&mut stdout()).unwrap();
}
