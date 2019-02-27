use std::collections::HashMap;
use crate::ast;
use crate::nast::*;

fn fresh_intermediate(intermediates: &mut HashMap<String, Expr>) -> String {
	let i = 1;
	loop {
		let name = format!("tmp{}", i);
		if !intermediates.contains_key(&name) {
			return name;
		}
	}
}

fn normalize_atom(e: &ast::Expr, intermediates: &mut HashMap<String, Expr>) -> Atom {
	match e {
		ast::Expr::Const(c) => Atom::Const(c.clone()),
		ast::Expr::Ident(ident) => Atom::Ident(ident.to_string()),
		_ => {
			// Create a local variable to store the intermediate value
			let name = fresh_intermediate(intermediates);
			let e = normalize_expr(e, intermediates);
			intermediates.insert(name.clone(), e);
			Atom::Ident(name)
		},
	}
}

fn normalize_bexpr(e: &ast::Expr, intermediates: &mut HashMap<String, Expr>) -> Bexpr {
	match e {
		ast::Expr::Unop(unop, e) => Bexpr::Unop(unop.clone(), Box::new(normalize_bexpr(e, intermediates))),
		ast::Expr::Binop(binop, exprs) => {
			let (e1, e2): &(ast::Expr, ast::Expr) = &*exprs;
			Bexpr::Binop(binop.clone(), Box::new((normalize_bexpr(e1, intermediates), normalize_bexpr(e2, intermediates))))
		},
		ast::Expr::If(iff) => {
			let (cond, body, else_part): &(ast::Expr, ast::Expr, ast::Expr) = &*iff;
			Bexpr::If(Box::new((normalize_bexpr(cond, intermediates), normalize_bexpr(body, intermediates), normalize_bexpr(else_part, intermediates))))
		},
		ast::Expr::Tuple(exprs) => Bexpr::Tuple(exprs.iter().map(|e| normalize_bexpr(e, intermediates)).collect()),
		_ => Bexpr::Atom(normalize_atom(e, intermediates)),
	}
}

fn normalize_expr(e: &ast::Expr, intermediates: &mut HashMap<String, Expr>) -> Expr {
	match e {
		ast::Expr::Call{name, args} => Expr::Call{
			name: name.to_string(),
			args: args.iter().map(|e| normalize_bexpr(e, intermediates)).collect(),
		},
		ast::Expr::Fby(fby) => {
			let (e1, e2): &(ast::Expr, ast::Expr) = &*fby;
			// TODO: extract tuples
			Expr::Fby(vec!(normalize_atom(e1, intermediates)), vec!(normalize_expr(e2, intermediates)))
		},
		_ => Expr::Bexpr(normalize_bexpr(e, intermediates)),
	}
}

fn normalize_equation(eq: &ast::Equation, intermediates: &mut HashMap<String, Expr>) -> Equation {
	Equation{
		names: eq.names.clone(),
		body: normalize_expr(&eq.body, intermediates),
	}
}

fn normalize_node(n: &ast::Node) -> Node {
	let mut intermediates = HashMap::new();
	// Prevent local names from being used for intermediates
	for (name, _) in n.locals.iter() {
		intermediates.insert(name.clone(), Expr::Bexpr(Bexpr::Atom(Atom::Const(Const::Unit))));
	}
	let mut body: Vec<Equation> = n.body.iter().map(|eq| {
		normalize_equation(eq, &mut intermediates)
	}).collect();
	let mut locals = n.locals.clone();
	for (name, e) in intermediates {
		// TODO: the local name isn't Type::Unit (though we don't use it)
		locals.insert(name.clone(), Type::Unit);
		body.push(Equation{names: vec!(name), body: e});
	}
	Node{
		name: n.name.clone(),
		args_in: n.args_in.clone(),
		args_out: n.args_out.clone(),
		locals: locals,
		body: body,
	}
}

pub fn normalize(f: &[ast::Node]) -> Vec<Node> {
	f.iter().map(normalize_node).collect()
}
