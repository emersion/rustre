use crate::ast;
use crate::nast::*;

fn normalize_atom(e: &ast::Expr) -> Atom {
	match e {
		ast::Expr::Const(c) => Atom::Const(c.clone()),
		ast::Expr::Ident(ident) => Atom::Ident(ident.to_string()),
		_ => unreachable!(), // TODO
	}
}

fn normalize_bexpr(e: &ast::Expr) -> Bexpr {
	match e {
		ast::Expr::Unop(unop, e) => Bexpr::Unop(unop.clone(), Box::new(normalize_bexpr(e))),
		ast::Expr::Binop(binop, exprs) => {
			let (e1, e2): &(ast::Expr, ast::Expr) = &*exprs;
			Bexpr::Binop(binop.clone(), Box::new((normalize_bexpr(e1), normalize_bexpr(e2))))
		},
		ast::Expr::If(iff) => {
			let (cond, body, else_part): &(ast::Expr, ast::Expr, ast::Expr) = &*iff;
			Bexpr::If(Box::new((normalize_bexpr(cond), normalize_bexpr(body), normalize_bexpr(else_part))))
		},
		ast::Expr::Tuple(exprs) => Bexpr::Tuple(exprs.iter().map(normalize_bexpr).collect()),
		_ => Bexpr::Atom(normalize_atom(e)),
	}
}

fn normalize_expr(e: &ast::Expr) -> Expr {
	match e {
		ast::Expr::Call{name, args} => Expr::Call{
			name: name.to_string(),
			args: args.iter().map(normalize_bexpr).collect(),
		},
		ast::Expr::Fby(fby) => {
			let (e1, e2): &(ast::Expr, ast::Expr) = &*fby;
			Expr::Fby(vec!(normalize_atom(e1)), vec!(normalize_atom(e2)))
		},
		_ => Expr::Bexpr(normalize_bexpr(e)),
	}
}

fn normalize_equation(eq: &ast::Equation) -> Equation {
	Equation{
		names: eq.names.clone(),
		body: normalize_expr(&eq.body),
	}
}

pub fn normalize(n: &ast::Node) -> Node {
	Node{
		name: n.name.clone(),
		args_in: n.args_in.clone(),
		args_out: n.args_out.clone(),
		locals: n.locals.clone(),
		body: n.body.iter().map(normalize_equation).collect(),
	}
}
