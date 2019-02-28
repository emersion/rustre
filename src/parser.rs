use std::collections::HashMap;
use pest::Parser;
use pest::iterators::Pair;
use pest::error::Error;
use crate::ast::*;

#[derive(Parser)]
#[grammar = "lustre.pest"]
pub struct LustreParser;

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
			arg_list.insert(arg, typ.clone());
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
		Rule::int => Const::Int(pair.as_str().parse::<i32>().unwrap()),
		Rule::float => Const::Float(pair.as_str().parse::<f32>().unwrap()),
		Rule::string => Const::String(pair.into_inner().next().unwrap().as_str().to_string()),
		Rule::unit => Const::Unit,
		_ => unreachable!(),
	}
}

fn parse_local(pair: Pair<Rule>) -> HashMap<String, Type> {
	assert!(pair.as_rule() == Rule::local);
	match pair.into_inner().next() {
		Some(v) => parse_arg_list(v),
		None => HashMap::new(),
	}
}

fn parse_unop(pair: Pair<Rule>) -> Unop {
	assert!(pair.as_rule() == Rule::unop);
	match pair.as_str() {
		"-" => Unop::Minus,
		"-." => Unop::MinusDot,
		"not" => Unop::Not,
		_ => unreachable!(),
	}
}

enum BinopOrFby {
	Binop(Binop),
	Fby,
}

fn parse_binop_or_fby(pair: Pair<Rule>) -> BinopOrFby {
	assert!(pair.as_rule() == Rule::binop);
	match pair.as_str() {
		"+" => BinopOrFby::Binop(Binop::Plus),
		"-" => BinopOrFby::Binop(Binop::Minus),
		"*" => BinopOrFby::Binop(Binop::Mult),
		"/" => BinopOrFby::Binop(Binop::Div),
		"+." => BinopOrFby::Binop(Binop::PlusDot),
		"-." => BinopOrFby::Binop(Binop::MinusDot),
		"*." => BinopOrFby::Binop(Binop::MultDot),
		"/." => BinopOrFby::Binop(Binop::DivDot),
		"<" => BinopOrFby::Binop(Binop::Lt),
		">" => BinopOrFby::Binop(Binop::Gt),
		"<=" => BinopOrFby::Binop(Binop::Leq),
		">=" => BinopOrFby::Binop(Binop::Geq),
		"=" => BinopOrFby::Binop(Binop::Eq),
		"and" => BinopOrFby::Binop(Binop::And),
		"or" => BinopOrFby::Binop(Binop::Or),
		"fby" => BinopOrFby::Fby,
		_ => unreachable!(),
	}
}

fn parse_term(pair: Pair<Rule>) -> Expr {
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
		Rule::unop_expr => {
			let mut inner_rules = pair.into_inner();
			let op = parse_unop(inner_rules.next().unwrap());
			let e = parse_expr(inner_rules.next().unwrap());
			Expr::Unop(op, Box::new(e))
		},
		Rule::ifrule => {
			let mut inner_rules = pair.into_inner();
			let cond = parse_expr(inner_rules.next().unwrap());
			let bif = parse_expr(inner_rules.next().unwrap());
			let belse = parse_expr(inner_rules.next().unwrap());
			Expr::If(Box::new((cond, bif, belse)))
		},
		Rule::ident => {
			let id = pair.as_str().to_string();
			Expr::Ident(id)
		},
		Rule::pexpr => {
			parse_expr(pair.into_inner().next().unwrap())
		},
		Rule::expr_tuple => {
			let exprs = pair.into_inner().map(parse_expr).collect();
			Expr::Tuple(exprs)
		},
		_ => unreachable!(),
	}
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
	assert!(pair.as_rule() == Rule::expr);
	let mut inner_rules = pair.into_inner();
	let left = parse_term(inner_rules.next().unwrap());
	match inner_rules.next() {
		Some(binop_pair) => {
			let binop_or_fby = parse_binop_or_fby(binop_pair);
			let right = parse_expr(inner_rules.next().unwrap());
			match binop_or_fby {
				BinopOrFby::Binop(binop) => Expr::Binop(binop, Box::new((left, right))),
				BinopOrFby::Fby => Expr::Fby(Box::new((left, right))),
			}
		},
		None => left,
	}
}

fn parse_eq(pair: Pair<Rule>) -> Equation {
	assert!(pair.as_rule() == Rule::eq);
	let mut inner_rules = pair.into_inner();
	Equation{
		names: inner_rules.next().unwrap().into_inner().map(|p| p.as_str().to_string()).collect(),
		body: parse_expr(inner_rules.next().unwrap()),
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
		locals: parse_local(inner_rules.next().unwrap()),
		body: parse_eq_list(inner_rules.next().unwrap()),
	}
}

fn parse_file(pair: Pair<Rule>) -> Vec<Node> {
	assert!(pair.as_rule() == Rule::node_list);
	pair.into_inner().map(parse_node).collect()
}

pub fn parse(input: &str) -> Result<Vec<Node>, Error<Rule>> {
	let mut pair = LustreParser::parse(Rule::file, input)?;
	eprintln!("{:?}", pair);
	Ok(parse_file(pair.next().unwrap()))
}
