use std::collections::HashMap;
use std::io::{Write, Result};
use crate::nast::*;
use crate::typer::type_of_const;

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

impl WriterTo for Atom {
	fn write_to(&self, w: &mut Write) -> Result<()> {
		match self {
			Atom::Const(c) => c.write_to(w),
			Atom::Ident(ident) => write!(w, "{}", ident),
		}
	}
}

impl WriterTo for Bexpr {
	fn write_to(&self, w: &mut Write) -> Result<()> {
		match self {
			Bexpr::Unop(op, e) => {
				write!(w, "{} ", match op {
					Unop::Minus | Unop::MinusDot => "-",
					Unop::Not => "!",
				})?;
				e.write_to(w)
			},
			Bexpr::Binop(op, exprs) => {
				let (e1, e2): &(Bexpr, Bexpr) = &*exprs;
				e1.write_to(w)?;
				write!(w, "{} ", match op {
					Binop::Plus | Binop::PlusDot => " +",
					Binop::Minus | Binop::MinusDot => " -",
					Binop::Mult | Binop::MultDot => " *",
					Binop::Div | Binop::DivDot => " /",
					Binop::Lt => " <",
					Binop::Gt => " >",
					Binop::Leq => " <=",
					Binop::Geq => " >=",
					Binop::Eq => " ==",
					Binop::And => " &&",
					Binop::Or => " ||",
				})?;
				e2.write_to(w)
			},
			Bexpr::If(iff) => {
				let (cond, body, else_part): &(Bexpr, Bexpr, Bexpr) = &*iff;
				write!(w, "if (")?;
				cond.write_to(w)?;
				write!(w, ") {{")?;
				body.write_to(w)?;
				write!(w, "}} else {{")?;
				else_part.write_to(w)?;
				write!(w, "}}")
			},
			Bexpr::Tuple(vex) => {
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
			},
			Bexpr::Atom(atom) => atom.write_to(w),
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
			Expr::Fby(_, _) => unreachable!(), // TODO
			Expr::Bexpr(bexp) => bexp.write_to(w),
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
		&self.body.write_to(w)?;
		write!(w, ";\n")
	}
}

fn get_type(t: Type) -> &'static str {
	match t {
		Type::Unit => "()",
		Type::Bool => "bool",
		Type::Int => "i32",
		Type::Float => "f32",
		Type::String => "String",
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
			write!(w, "{}", get_type(*typ))?;
		}
		if !first {
			write!(w, ", ")?;
		}
		first = false;
	}
	Ok(())
}

fn capitalize(s: &str) -> String {
	let mut c = s.chars();
	match c.next() {
		None => String::new(),
		Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
	}
}

fn format_struct(w: &mut Write, name: &str, fields: &HashMap<String, Type>) -> Result<()> {
	write!(w, "#[derive(Debug, Default)]\n")?;
	write!(w, "struct {} {{\n", name)?;
	for (k, t) in fields {
		write!(w, "\t{}: {},\n", k, get_type(*t))?;
	}
	write!(w, "}}\n\n")
}

struct NodeMemory {
	name: String,
	fields: HashMap<String, Type>,
}

fn get_node_mem(n: &Node) -> NodeMemory {
	let mut fields = HashMap::new();
	for eq in n.body.iter() {
		if let Expr::Fby(init, _) = &eq.body {
			// TODO: support tuples
			assert!(eq.names.len() == 1);
			let t = match &init[0] {
				Atom::Const(c) => type_of_const(c),
				Atom::Ident(_) => unreachable!(),
			};
			fields.insert(eq.names[0].clone(), t);
		}
	}
	NodeMemory{
		name: format!("Mem{}", capitalize(&n.name)),
		fields: fields,
	}
}

impl WriterTo for Node {
	fn write_to(&self, w: &mut Write) -> Result<()> {
		let mem = get_node_mem(self);
		if mem.fields.len() > 0 {
			format_struct(w, &mem.name, &mem.fields)?;
		}

		write!(w, "fn {}(", &self.name)?;
		if mem.fields.len() > 0 {
			write!(w, "mem: &mut {}, ", &mem.name)?;
		}
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
				Bexpr::Atom(Atom::Const(c))
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
