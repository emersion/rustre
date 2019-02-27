use std::collections::HashMap;
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
        }
        Bexpr::Tuple(vexpr) => { // may be improved in some cases
            let v = vexpr.iter().map(find_dep_bexpr);
            v.into_iter().flatten().collect()
        }
        Bexpr::Atom(a) => find_dep_atom(a),
    }
}

// Finds the direct dependencies to compute the equation
fn find_dep_eq(e: &Equation) -> Vec<String> {
    match &e.body {
        Expr::Bexpr(be) => find_dep_bexpr(&be),
        Expr::Call{name, args} => {
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

fn sequentialize_node(n: &Node) -> Node {
    // create dependency graph : HashMap<String, Vec<String>>
    let mut deps = HashMap::new();
    for eq in &n.body {
        let dep = find_dep_eq(&eq);
        for name in &eq.names {
            if deps.contains_key(&name) {
                panic!("Two equations define '{}' in node '{}'", &name, &n.name)
            }
            deps.insert(name, dep.clone());
        }
    }

    // Resolve dependencies

    Node{
        name: n.name.clone(),
        args_in: n.args_in.clone(),
        args_out: n.args_out.clone(),
        locals: n.locals.clone(),
        body: vec!{},
    }
}

pub fn sequentialize(f: &[Node]) -> Vec<Node> {
    f.iter().map(sequentialize_node).collect()
}
