use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Type {
	Unit,
	Bool,
	Int,
	Float,
	String,
}

#[derive(Debug)]
pub enum Const {
	Unit,
	Bool(bool),
	Int(i32),
	Float(f32),
	String(String),
}

#[derive(Debug)]
pub enum Unop {
	Minus,
	MinusDot,
	Not,
}

#[derive(Debug)]
pub enum Binop {
	Plus,
	Minus,
	Mult,
	Div,
	PlusDot,
	MinusDot,
	MultDot,
	DivDot,
	Lt,
	Gt,
	Leq,
	Geq,
	Eq,
	And,
	Or,
	Fby,
}

#[derive(Debug)]
pub enum Expr {
	Call{
		name: String,
		args: Vec<Expr>,
	},
	Const(Const),
	Unop(Unop, Box<Expr>),
	Binop(Binop, Box<(Expr, Expr)>),
	If(Box<(Expr, Expr, Expr)>),
	Ident(String),
	Tuple(Vec<Expr>),
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
