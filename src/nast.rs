// Normalized AST

use std::collections::HashMap;
pub use crate::ast::{Type, Const, Unop, Binop};

#[derive(Debug)]
pub enum Atom {
	Ident(String),
	Const(Const),
}

#[derive(Debug)]
pub enum Bexpr {
	Atom(Atom),
	Unop(Unop, Box<Bexpr>),
	Binop(Binop, Box<(Bexpr, Bexpr)>),
	If(Box<(Bexpr, Bexpr, Bexpr)>),
	Tuple(Vec<Bexpr>),
}

#[derive(Debug)]
pub enum Expr {
	Bexpr(Bexpr),
	Call{
		name: String,
		args: Vec<Bexpr>,
	},
	Fby(Vec<Atom>, Vec<Atom>),
}

#[derive(Debug)]
pub struct Equation {
	pub names: Vec<String>,
	pub body: Expr,
}

#[derive(Debug)]
pub struct Node {
	pub name: String,
	pub args_in: HashMap<String, Type>,
	pub args_out: HashMap<String, Type>,
	pub locals: HashMap<String, Type>,
	pub body: Vec<Equation>,
}