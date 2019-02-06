extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ast;

use pest::Parser;
use std::collections::HashMap;
use crate::ast::*;

#[derive(Parser)]
#[grammar = "lustre.pest"]
pub struct LustreParser;

fn main() {
	let successful_src = "node abc() returns (o, p: unit); let o = print(\"hello world\"); tel";
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
			_ => panic!("invalid type: {}", pair.as_str())
		}
	}

	fn parse_arg(pair: Pair<Rule>) -> (Vec<String>, Type) {
		assert!(pair.as_rule() == Rule::arg);

		let mut inner_rules = pair.into_inner();
		(
			inner_rules.next().unwrap().into_inner().map(|p| p.as_str().to_string()).collect(),
			parse_type(inner_rules.next().unwrap()),
		)
	}

	fn parse_arg_list(pair: Pair<Rule>) -> HashMap<String, Type> {
		assert!(pair.as_rule() == Rule::arg_list);
		let mut arg_list = HashMap::new();
		for arg_idents in pair.into_inner() {
			let (args, typ) = parse_arg(arg_idents);
			for arg in args {
				arg_list.insert(arg, typ);
			}
		}
		arg_list
	}

	fn parse_constant(pair: Pair<Rule>) -> Const {
		match pair.as_rule() {
			Rule::bool => match pair.as_str() {
				"true" => Const::Bool(true),
				"false" => Const::Bool(false),
				_ => unreachable!(),
			},
			Rule::int => unreachable!(), // TODO
			Rule::string => Const::String(pair.into_inner().next().unwrap().as_str().to_string()),
			_ => unreachable!(),
		}
	}

	fn parse_expr(pair: Pair<Rule>) -> Expr {
		match pair.as_rule() {
			Rule::call => {
				let mut inner_rules = pair.into_inner();
				Expr::Call{
					name: inner_rules.next().unwrap().as_str().to_string(),
					args: inner_rules.map(parse_expr).collect(),
				}
			},
			Rule::constant => {
				let c = parse_constant(pair.into_inner().next().unwrap());
				Expr::Const(c)
			},
			_ => unreachable!(),
		}
	}

	fn parse_eq(pair: Pair<Rule>) -> Equation {
		assert!(pair.as_rule() == Rule::eq);

		let mut inner_rules = pair.into_inner();
		Equation{
			name: inner_rules.next().unwrap().as_str().to_string(),
			value: parse_expr(inner_rules.next().unwrap()),
		}
	}

	fn parse_eq_list(pair: Pair<Rule>) -> Vec<Equation> {
		assert!(pair.as_rule() == Rule::eq_list);
		pair.into_inner().map(parse_eq).collect()
	}

	fn parse_node(pair: Pair<Rule>) -> Node {
		assert!(pair.as_rule() == Rule::node);

		let mut inner_rules = pair.into_inner();
		Node{
			name: inner_rules.next().unwrap().as_str().to_string(),
			args_in: parse_arg_list(inner_rules.next().unwrap()),
			args_out: parse_arg_list(inner_rules.next().unwrap()),
			body: parse_eq_list(inner_rules.next().unwrap()),
		}
	}

	fn parse_file(pair: Pair<Rule>) -> Vec<Node> {
		assert!(pair.as_rule() == Rule::node_list);
		pair.into_inner().map(parse_node).collect()
	}

	println!("{:?}", parse_file(successful_parse.unwrap().next().unwrap()));
}
