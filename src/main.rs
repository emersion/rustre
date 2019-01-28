extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "lustre.pest"]
pub struct LustreParser;

#[derive(Debug)]
enum Type {
	TypeUnit,
	TypeBool,
	TypeInt,
	TypeFloat,
	TypeString,
}

#[derive(Debug)]
enum Const {
	ConstString(String),
	ConstInt(i32),
	ConstFloat(f32),
}

#[derive(Debug)]
enum Expr {
	ExprCall{
		name: String,
		args: Vec<Expr>,
	},
	ExprConst(Const),
}

#[derive(Debug)]
struct Equation {
	name: String,
	value: Expr,
}

#[derive(Debug)]
struct Node {
	name: String,
	args_in: HashMap<String, Type>,
	args_out: HashMap<String, Type>,
	body: Vec<Equation>,
}

fn main() {
	let successful_src = "node abc() returns (o: unit); let o = print(\"hello world\"); tel";
	let successful_parse = LustreParser::parse(Rule::file, successful_src);
	println!("{:?}", successful_parse);

	let unsuccessful_parse = LustreParser::parse(Rule::file, "this is not a Lustre program");
	println!("{:?}", unsuccessful_parse);

	use pest::iterators::Pair;

	fn parse_node(pair: Pair<Rule>) -> Node {
		match pair.as_rule() {
			// TODO
			Rule::node => Node{
				name: pair.into_inner().next().unwrap().to_string(),
				args_in: HashMap::new(),
				args_out: HashMap::new(),
				body: Vec::new(),
			},
			_ => unreachable!(),
		}
	}

	fn parse_file(pair: Pair<Rule>) -> Vec<Node> {
		println!("{:?}", pair.as_rule());
		match pair.as_rule() {
			Rule::node_list => pair.into_inner().map(parse_node).collect(),
			_ => unreachable!(),
		}
	}

	println!("{:?}", parse_file(successful_parse.unwrap().next().unwrap()));
}
