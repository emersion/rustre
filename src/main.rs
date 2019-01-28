extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Parser)]
#[grammar = "lustre.pest"]
pub struct LustreParser;

#[derive(Debug)]
enum Type {
	Unit,
	Bool,
	Int,
	Float,
	String,
}

#[derive(Debug)]
enum Const {
	String(String),
	Int(i32),
	Float(f32),
}

#[derive(Debug)]
enum Expr {
	Call{
		name: String,
		args: Vec<Expr>,
	},
	Const(Const),
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

	fn parse_type(pair: Pair<Rule>) -> Type {
		match pair.as_str() {
			"unit" => Type::Unit,
			"bool" => Type::Bool,
			"int" => Type::Int,
			"float" => Type::Float,
			"string" => Type::String,
			_ => unreachable!(), // TODO: better error message
		}
	}

	fn parse_arg(pair: Pair<Rule>) -> (String, Type) {
		match pair.as_rule() {
			Rule::arg => {
				let mut inner_rules = pair.into_inner();
				(
					inner_rules.next().unwrap().as_str().to_string(),
					parse_type(inner_rules.next().unwrap()),
				)
			},
			_ => unreachable!(),
		}
	}

	fn parse_arg_list(pair: Pair<Rule>) -> HashMap<String, Type> {
		match pair.as_rule() {
			Rule::arg_list => {
				HashMap::from_iter(pair.into_inner().map(parse_arg))
			},
			_ => unreachable!(),
		}
	}

	fn parse_eq_list(pair: Pair<Rule>) -> Vec<Equation> {
		Vec::new() // TODO
	}

	fn parse_node(pair: Pair<Rule>) -> Node {
		match pair.as_rule() {
			Rule::node => {
				let mut inner_rules = pair.into_inner();
				Node{
					name: inner_rules.next().unwrap().as_str().to_string(),
					args_in: parse_arg_list(inner_rules.next().unwrap()),
					args_out: parse_arg_list(inner_rules.next().unwrap()),
					body: parse_eq_list(inner_rules.next().unwrap()),
				}
			},
			_ => unreachable!(),
		}
	}

	fn parse_file(pair: Pair<Rule>) -> Vec<Node> {
		match pair.as_rule() {
			Rule::node_list => pair.into_inner().map(parse_node).collect(),
			_ => unreachable!(),
		}
	}

	println!("{:?}", parse_file(successful_parse.unwrap().next().unwrap()));
}
