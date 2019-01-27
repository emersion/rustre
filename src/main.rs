extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "lustre.pest"]
pub struct LustreParser;

fn main() {
	let successful_parse = LustreParser::parse(Rule::file, "node abc() returns (o: unit); let o = print(\"hello world\"); tel");
	println!("{:?}", successful_parse);

	let unsuccessful_parse = LustreParser::parse(Rule::file, "this is not a Lustre program");
	println!("{:?}", unsuccessful_parse);
}
