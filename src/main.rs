extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;
mod nast;
mod normalizer;
mod parser;
mod rustfmt;
mod sequentializer;
mod typer;

use crate::normalizer::normalize;
use crate::parser::parse;
use crate::rustfmt::format;
use crate::sequentializer::sequentialize;
use std::io::{stdin, stdout, Read};

fn main() {
    let mut buffer = String::new();
    stdin().read_to_string(&mut buffer).unwrap();

    let f = parse(&buffer).unwrap();
    eprintln!("{:?}", &f);

    let nf = normalize(&f);
    eprintln!("{:?}", &nf);

    let sf = sequentialize(&nf);
    eprintln!("{:?}", &sf);

    format(&mut stdout(), &sf).unwrap();
}
