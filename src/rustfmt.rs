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
			Expr::UnopExpr(_, _) => unreachable!(), // TODO
		}
	}
}

impl WriterTo for Equation {
	fn write_to(&self, w: &mut Write) -> Result<()> {
		write!(w, "\tlet {} = ", &self.name)?;
		&self.value.write_to(w)?;
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

		write!(w, "fn main() {{}}\n")
	}
}
