use crate::nast::*;

// TODO: this doesn't perform type checking

pub fn type_of_const(c: &Const) -> Type {
    match c {
        Const::Unit => Type::Unit,
        Const::Bool(_) => Type::Bool,
        Const::Int(_) => Type::Int,
        Const::Float(_) => Type::Float,
        Const::String(_) => Type::String,
    }
}

fn type_of_atom(a: &Atom) -> Type {
    match a {
        Atom::Const(c) => type_of_const(c),
        Atom::Ident(_) => unreachable!(), // TODO
    }
}

fn type_of_bexpr(e: &Bexpr) -> Type {
    match e {
        Bexpr::Unop(op, _) => match op {
            Unop::Minus => Type::Int,
            Unop::MinusDot => Type::Float,
            Unop::Not => Type::Bool,
        },
        Bexpr::Binop(op, _) => match op {
            Binop::Plus | Binop::Minus | Binop::Mult | Binop::Div => Type::Int,
            Binop::PlusDot | Binop::MinusDot | Binop::MultDot | Binop::DivDot => Type::Float,
            Binop::Lt
            | Binop::Gt
            | Binop::Leq
            | Binop::Geq
            | Binop::Eq
            | Binop::And
            | Binop::Or => Type::Bool,
        },
        Bexpr::If(iff) => {
            let (_, body, _): &(Bexpr, Bexpr, Bexpr) = &*iff;
            type_of_bexpr(body)
        }
        Bexpr::Tuple(_) => unreachable!(), // TODO
        Bexpr::Atom(atom) => type_of_atom(atom),
    }
}

pub fn type_of(e: &Expr) -> Type {
    match e {
        Expr::Call { name: _, args: _ } => unreachable!(), // TODO
        Expr::Fby(_, _) => unreachable!(),                 // TODO
        Expr::Bexpr(bexp) => type_of_bexpr(bexp),
    }
}
