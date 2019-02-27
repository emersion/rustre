use crate::nast::*;
use crate::typer::type_of_const;
use std::collections::HashMap;
use std::io::{Result, Write};

fn format_const(w: &mut Write, c: &Const) -> Result<()> {
	match c {
		Const::Unit => write!(w, "()"),
		Const::Bool(b) => write!(w, "{}", b),
		Const::Int(i) => write!(w, "{}", i),
		Const::Float(f) => write!(w, "{}", f),
		Const::String(s) => write!(w, "\"{}\"", s), // TODO: escaping
	}
}

fn format_atom(w: &mut Write, atom: &Atom) -> Result<()> {
	match atom {
		Atom::Const(c) => format_const(w, c),
		Atom::Ident(ident) => write!(w, "{}", ident),
	}
}

fn format_bexpr(w: &mut Write, bexpr: &Bexpr) -> Result<()> {
	match bexpr {
		Bexpr::Unop(op, e) => {
			write!(
				w,
				"{} ",
				match op {
					Unop::Minus | Unop::MinusDot => "-",
					Unop::Not => "!",
				}
			)?;
			format_bexpr(w, e)
		}
		Bexpr::Binop(op, exprs) => {
			let (e1, e2): &(Bexpr, Bexpr) = &*exprs;
			format_bexpr(w, e1)?;
			write!(
				w,
				"{} ",
				match op {
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
				}
			)?;
			format_bexpr(w, e2)
		}
		Bexpr::If(iff) => {
			let (cond, body, else_part): &(Bexpr, Bexpr, Bexpr) = &*iff;
			write!(w, "if (")?;
			format_bexpr(w, cond)?;
			write!(w, ") {{")?;
			format_bexpr(w, body)?;
			write!(w, "}} else {{")?;
			format_bexpr(w, else_part)?;
			write!(w, "}}")
		}
		Bexpr::Tuple(vex) => {
			match vex.split_first() {
				Some((fst, elems)) => {
					write!(w, "(")?;
					format_bexpr(w, fst)?;
					for e in elems {
						// skipping #1
						write!(w, ", ")?;
						format_bexpr(w, e)?;
					}
					write!(w, ")")
				}
				None => unreachable!(),
			}
		}
		Bexpr::Atom(atom) => format_atom(w, atom),
	}
}

fn format_expr(
	w: &mut Write,
	e: &Expr,
	dest: &str,
	mems: &HashMap<String, NodeMemory>,
) -> Result<()> {
	match e {
		Expr::Call { name, args } => {
			write!(w, "{}(", name)?;
			if mems.get(name).is_some() {
				if dest == "" {
					// Used in main()
					write!(w, "&mut mem, ")?;
				} else {
					write!(w, "&mut mem.{}, ", dest)?;
				}
			}
			let mut first = true;
			for arg in args {
				format_bexpr(w, arg)?;
				if !first {
					write!(w, ", ")?;
				}
				first = false;
			}
			write!(w, ")")
		}
		Expr::Fby(_, _) => write!(w, "mem.{}", dest),
		Expr::Bexpr(bexpr) => format_bexpr(w, bexpr),
	}
}

fn format_equation(w: &mut Write, eq: &Equation, mems: &HashMap<String, NodeMemory>) -> Result<()> {
	write!(w, "\tlet ")?;
	let (fst, elems) = (&eq.names).split_first().unwrap();
	if !elems.is_empty() {
		write!(w, "(")?;
	}
	write!(w, "{}", fst)?;
	for e in elems {
		write!(w, ", {}", e)?;
	}
	if !elems.is_empty() {
		write!(w, ")")?;
	}
	write!(w, " = ")?;
	// TODO: support tuples
	format_expr(w, &eq.body, fst, mems)?;
	write!(w, ";\n")
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

fn format_arg_list(
	w: &mut Write,
	args: &HashMap<String, Type>,
	with_name: bool,
	with_typ: bool,
) -> Result<()> {
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

fn format_struct(
	w: &mut Write,
	name: &str,
	fields: &HashMap<String, String>,
	init_values: &HashMap<String, Const>,
) -> Result<()> {
	write!(w, "#[derive(Debug)]\n")?;
	write!(w, "struct {} {{\n", name)?;
	for (k, t) in fields {
		write!(w, "\t{}: {},\n", k, t)?;
	}
	write!(w, "}}\n\n")?;

	write!(w, "impl Default for {} {{\n", name)?;
	write!(w, "\tfn default() -> Self {{\n")?;
	write!(w, "\t\tSelf{{\n")?;
	for k in fields.keys() {
		write!(w, "\t\t\t{}: ", k)?;
		match init_values.get(k) {
			Some(c) => format_const(w, c)?,
			None => write!(w, "Default::default()")?,
		}
		write!(w, ",\n")?;
	}
	write!(w, "\t\t}}\n")?;
	write!(w, "\t}}\n")?;
	write!(w, "}}\n\n")
}

struct NodeMemory {
	name: String,
	fields: HashMap<String, String>,
	init_values: HashMap<String, Const>,
	next_values: HashMap<String, Expr>,
}

fn get_node_mem(n: &Node, mems: &HashMap<String, NodeMemory>) -> Option<NodeMemory> {
	let mut fields = HashMap::new();
	let mut init_values = HashMap::new();
	let mut next_values = HashMap::new();
	for eq in n.body.iter() {
		// TODO: support tuples
		assert!(eq.names.len() == 1);
		let dest = &eq.names[0];

		match &eq.body {
			Expr::Call { name, .. } => {
				if let Some(call_mem) = mems.get(name) {
					fields.insert(dest.clone(), call_mem.name.clone());
				}
			}
			Expr::Fby(init, next) => {
				// TODO: support tuples
				let (init, next) = (&init[0], &next[0]);
				let init = match init {
					Atom::Const(c) => c,
					Atom::Ident(_) => unreachable!(),
				};
				let t = type_of_const(init);
				init_values.insert(dest.clone(), init.clone());
				next_values.insert(dest.clone(), next.clone());
				fields.insert(dest.clone(), get_type(t).to_string());
			}
			_ => {}
		}
	}

	if fields.is_empty() {
		None
	} else {
		Some(NodeMemory {
			name: format!("Mem{}", capitalize(&n.name)),
			fields,
			init_values,
			next_values,
		})
	}
}

fn format_node(w: &mut Write, n: &Node, mems: &HashMap<String, NodeMemory>) -> Result<()> {
	let mem = mems.get(&n.name);
	if let Some(mem) = mem {
		format_struct(w, &mem.name, &mem.fields, &mem.init_values)?;
	}

	write!(w, "fn {}(", &n.name)?;
	if let Some(mem) = mem {
		write!(w, "mem: &mut {}, ", &mem.name)?;
	}
	format_arg_list(w, &n.args_in, true, true)?;
	write!(w, ") -> (")?;
	format_arg_list(w, &n.args_out, false, true)?;
	write!(w, ") {{\n")?;
	for eq in &n.body {
		format_equation(w, eq, mems)?;
	}

	if let Some(mem) = mem {
		for (k, v) in &mem.next_values {
			write!(w, "\tmem.{} = ", k)?;
			format_expr(w, v, "_", mems)?;
			write!(w, ";\n")?;
		}
	}

	write!(w, "\treturn (")?;
	format_arg_list(w, &n.args_out, true, false)?;
	write!(w, ");\n")?;
	write!(w, "}}\n\n")
}

pub fn format(w: &mut Write, f: &[Node]) -> Result<()> {
	write!(w, "fn print(s: &str) {{\n")?;
	write!(w, "\tprintln!(\"{{}}\", s);\n")?;
	write!(w, "}}\n\n")?;

	let mut mems = HashMap::new();
	for n in f {
		if let Some(mem) = get_node_mem(n, &mems) {
			mems.insert(n.name.clone(), mem);
		}
	}

	for n in f {
		format_node(w, n, &mems)?;
	}

	// Call the last node in main()
	write!(w, "fn main() {{\n")?;
	if let Some(n) = f.last() {
		// Pick some initial values for the node
		// TODO: we should probably ask these to the user, and run the node in a loop
		let argv = n
			.args_in
			.iter()
			.map(|(_name, typ)| {
				let c = match typ {
					Type::Unit => Const::Unit,
					Type::Int => Const::Int(42),
					_ => unreachable!(), // TODO
				};
				Bexpr::Atom(Atom::Const(c))
			})
			.collect();
		let call = Expr::Call {
			name: n.name.clone(),
			args: argv,
		};

		if let Some(call_mem) = mems.get(&n.name) {
			write!(
				w,
				"\tlet mut mem: {} = Default::default();\n",
				&call_mem.name
			)?;
		}

		write!(w, "\tfor _ in 0..10 {{\n")?;

		write!(w, "\t\tlet v = ")?;
		format_expr(w, &call, "", &mems)?;
		write!(w, ";\n")?;

		write!(w, "\t\teprintln!(\"{{:?}}\", &v);\n")?;

		write!(w, "\t}}\n")?;
	}
	write!(w, "}}\n")
}
