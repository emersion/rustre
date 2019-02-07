use std::collections::HashMap;
use std::io::{Write, Result};
use crate::ast::*;

pub trait WriterTo {
	fn write_to(&self, w: &mut Write) -> Result<()>;
}

impl WriterTo for Const {
	fn write_to(&self, w: &mut Write) -> Result<()> {
		match self {
			Const::Unit => write!(w, "()"),
			Const::Bool(b) => write!(w, "{}", b),
			Const::Int(i) => write!(w, "{}", i),
			Const::Float(f) => write!(w, "{}", f),
			Const::String(s) => write!(w, "\"{}\"", s), // TODO: escaping
		}
	}
}

impl WriterTo for Expr {
	fn write_to(&self, w: &mut Write) -> Result<()> {
		match self {
			Expr::Call{name, args} => {
				write!(w, "{}(", name)?;
				let mut first = true;
				for arg in args {
					arg.write_to(w)?;
					if !first {
						write!(w, ", ")?;
					}
					first = false;
				}
				write!(w, ")")
			},
			Expr::Const(c) => c.write_to(w),
			Expr::Unop(op, e) => {
				write!(w, "{} ", match op {
					Unop::Minus | Unop::MinusDot => "-",
					Unop::Not => "!",
				})?;
				e.write_to(w)
			},
			Expr::Binop(op, exprs) => {
				let (e1, e2): &(Expr, Expr) = &*exprs;
				e1.write_to(w)?;
				write!(w, "{} ", match op {
					Binop::Plus | Binop::PlusDot => "+",
					Binop::Minus | Binop::MinusDot => "-",
					Binop::Mult | Binop::MultDot => "*",
					Binop::Div | Binop::DivDot => "/",
					Binop::Lt => "<",
					Binop::Gt => ">",
					Binop::Leq => "<=",
					Binop::Geq => ">=",
					Binop::Eq => "==",
					Binop::And => "&&",
					Binop::Or => "||",
					Binop::Fby => "fby",
				})?;
				e2.write_to(w)
			},
			Expr::Ident(ident) => write!(w, "{}", ident),
			Expr::If(iff) => {
				let (cond, body, else_part): &(Expr, Expr, Expr) = &*iff;
				write!(w, "if (")?;
				cond.write_to(w)?;
				write!(w, ") {{")?;
				body.write_to(w)?;
				write!(w, "}} else {{")?;
				else_part.write_to(w)?;
				write!(w, "}}")
			},
			Expr::Tuple(vex) => {
				match vex.split_first() {
					Some((fst, elems)) => {
						write!(w, "(")?;
						fst.write_to(w)?;
						// elems.map(|e| { write!(w, ", ")?; e.write_to(w)? }); Vec not designed that way
						for e in elems { // skipping #1
							write!(w, ", ")?;
							e.write_to(w)?;
						}
						write!(w, ")")
					},
					None => unreachable!(),
				}
			}
		}
	}
}

impl WriterTo for Equation {
	fn write_to(&self, w: &mut Write) -> Result<()> {
		write!(w, "\tlet ")?;
		let (fst, elems) = (&self.names).split_first().unwrap();
		if !elems.is_empty()  { write!(w, "(")?; }
		write!(w, "{}", fst)?;
		for e in elems {
			write!(w, ", {}", e)?;
		}
		if !elems.is_empty()  { write!(w, ")")?; }
		write!(w, " = ")?;
		&self.values.write_to(w)?;
		write!(w, ";\n")
	}
}

fn format_arg_list(w: &mut Write, args: &HashMap<String, Type>, with_name: bool, with_typ: bool) -> Result<()> {
	let mut first = true;
	for (name, typ) in args {
		if with_name {
			write!(w, "{}", name)?;
		}
		if with_name && with_typ {
			write!(w, ": ")?;
		}
		if with_typ {
			write!(w, "{}", match typ {
				Type::Unit => "()",
				Type::Bool => "bool",
				Type::Int => "i32",
				Type::Float => "f32",
				Type::String => "String",
			})?;
		}
		if !first {
			write!(w, ", ")?;
		}
		first = false;
	}
	Ok(())
}

impl WriterTo for Node {
	fn write_to(&self, w: &mut Write) -> Result<()> {
		write!(w, "fn {}(", &self.name)?;
		format_arg_list(w, &self.args_in, true, true)?;
		write!(w, ") -> (")?;
		format_arg_list(w, &self.args_out, false, true)?;
		write!(w, ") {{\n")?;
		for eq in &self.body {
			&eq.write_to(w)?;
		}
		write!(w, "\treturn (")?;
		format_arg_list(w, &self.args_out, true, false)?;
		write!(w, ");\n")?;
		write!(w, "}}\n\n")
	}
}

impl WriterTo for Vec<Node> {
	fn write_to(&self, w: &mut Write) -> Result<()> {
		write!(w, "fn print(s: &str) {{\n")?;
		write!(w, "\tprintln!(\"{{}}\", s);\n")?;
		write!(w, "}}\n\n")?;

		for n in self {
			&n.write_to(w)?;
		}

		// Call the last node in main()
		write!(w, "fn main() {{\n")?;
		if let Some(n) = self.last() {
			// Pick some initial values for the node
			// TODO: we should probably ask these to the user, and run the node in a loop
			let argv = n.args_in.iter().map(|(_name, typ)| {
				let c = match typ {
					Type::Unit => Const::Unit,
					Type::Int => Const::Int(42),
					_ => unreachable!(), // TODO
				};
				Expr::Const(c)
			}).collect();
			let call = Expr::Call{
				name: n.name.clone(),
				args: argv,
			};
			write!(w, "\t")?;
			call.write_to(w)?;
			write!(w, ";\n")?;
		}
		write!(w, "}}\n")
	}
}
