use std::collections::HashMap;
use std::collections::VecDeque;
use crate::nast::*;

fn find_dep_atom(a: &Atom) -> Vec<String> {
	match a {
		Atom::Ident(s) => vec!{s.to_string()},
		Atom::Const(_) => vec!{},
	}
}

fn find_dep_bexpr(e: &Bexpr) -> Vec<String> {
	match e {
		Bexpr::Unop(_, e) => find_dep_bexpr(e),
		Bexpr::Binop(_, exprs) => {
			let (e1, e2): &(Bexpr, Bexpr) = &*exprs;
			let mut v1 = find_dep_bexpr(e1);
			v1.append(&mut find_dep_bexpr(e2));
			v1
		},
		Bexpr::If(exprs) => {
			let (_, e1, e2): &(Bexpr, Bexpr, Bexpr) = &*exprs;
			let mut v1 = find_dep_bexpr(e1);
			v1.append(&mut find_dep_bexpr(e2));
			v1
		},
		Bexpr::Tuple(vexpr) => { // may be improved in some cases (ie: Tuple = Tuple)
			let v = vexpr.iter().map(find_dep_bexpr);
			v.into_iter().flatten().collect()
		},
		Bexpr::Atom(a) => find_dep_atom(a),
	}
}

// Finds the direct dependencies to compute the equation
fn find_dep_eq(e: &Equation) -> Vec<String> {
	match &e.body {
		Expr::Bexpr(be) => find_dep_bexpr(&be),
		Expr::Call{name:_, args} => {
			let v = args.iter().map(find_dep_bexpr);
			v.into_iter().flatten().collect()
		},
		Expr::Fby(vexpr1, vexpr2) => {
			if vexpr1.len() != vexpr2.len() {
				panic!("Expected same tuple size on fby")
			}
			let v = vexpr1.iter().map(find_dep_atom);
			v.into_iter().flatten().collect()
		},
	}
}

// propagates the dependencies for each equation
fn propagate(deps: &HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
	let mut finaldeps = HashMap::new();

	for (key, values) in deps {
		let mut todo = VecDeque::from(values.clone());
		let mut alldeps = vec!{};

		// while the queue [todo] is not empty
		while !todo.is_empty() {
			let d = todo.pop_front().unwrap();
			if deps.contains_key(&d) {
				let values = deps.get(&d).unwrap();
				for dnext in values {
					// don't add if already done or to be done
					if !alldeps.contains(dnext) && !todo.contains(dnext) {
						todo.push_back(dnext.to_string()); // add the dependecies
					}
				}
			}
			if !alldeps.contains(&d) { // adds the current value as done
				alldeps.push(d);
			}
		}
		finaldeps.insert(key.to_string(), alldeps);
	}
	finaldeps
}

fn order(n: &Node, mut alldeps: HashMap<String, Vec<String>>) -> Vec<Equation> {
	let mut orderedEqs = Vec::new();

	while !alldeps.is_empty() {
		let mut remove = Vec::new();

		for (var, deps) in &alldeps {
			let mut ok = true;

			for dep in deps {
				let isPrevEq = orderedEqs.iter().fold(false, |r, val: &Equation| {
					r || val.names.contains(&dep)
				});
				ok = ok && (n.args_in.contains_key(dep) || isPrevEq);
			}
			if ok { // dependencies satisfied
				let eq = n.body.iter().find(|&eq1| eq1.names.contains(&var)).unwrap();
				orderedEqs.push(eq.clone());

				// removing variables that are also computed by the equation
				for (k, _) in &alldeps {
					if eq.names.contains(k) {
						remove.push(k.clone());
					}
				}
			}
		}

		for k in &remove {
			alldeps.remove(k);
		}
	}
	orderedEqs
}

fn sequentialize_node(n: &Node) -> Node {
	// Create dependency graph
	let mut deps: HashMap<String, Vec<String>> = HashMap::new();
	for eq in &n.body {
		let dep = find_dep_eq(&eq);
		for name in &eq.names {
			if deps.contains_key(name) {
				panic!("Two equations define '{}' in node '{}'", name, &n.name)
			}
			deps.insert(name.clone(), dep.clone());
		}
	}

	let alldeps = propagate(&deps);

	eprintln!("Depedencies for Node [{}]", n.name);
	for (k, v) in &alldeps {
		eprintln!("{} -> {:?}", k, v)
	}

	// Check if there is a solution to the ordering problem
	for (key, deps) in &alldeps {
		if deps.contains(key) {
			panic!("Circular dependency detected for {} -> {:?}", key, deps)
		}
	}

	let orderedBody = order(&n, alldeps);

	Node{
		name: n.name.clone(),
		args_in: n.args_in.clone(),
		args_out: n.args_out.clone(),
		locals: n.locals.clone(),
		body: orderedBody,
	}
}

pub fn sequentialize(f: &[Node]) -> Vec<Node> {
	f.iter().map(sequentialize_node).collect()
}
