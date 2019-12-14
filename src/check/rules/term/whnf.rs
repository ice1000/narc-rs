use voile_util::loc::Ident;

use crate::check::monad::{ValTCM, TCE, TCM, TCS};
use crate::syntax::common::{ConHead, Ductive};
use crate::syntax::core::{Clause, Decl, Elim, Term, Val};

pub fn simplify(tcs: TCS, term: Term) -> ValTCM {
    match term {
        Term::Whnf(whnf) => Ok((whnf, tcs)),
        Term::Redex(def, id, elims) => match tcs.def(def) {
            Decl::Data(_) => Ok((Val::inductive(def, elims_to_terms(elims)?), tcs)),
            Decl::Codata(_) => Ok((Val::coinductive(def, elims_to_terms(elims)?), tcs)),
            Decl::Cons(cons) => {
                let (ductive, fields) = match &cons.fields {
                    Some(fields) => (Ductive::In, fields.clone()),
                    None => (Ductive::Coin, vec![]),
                };
                let head = ConHead::new(id, cons.data, ductive, fields);
                Ok((Val::Cons(head, elims_to_terms(elims)?), tcs))
            }
            Decl::Proj { .. } => unimplemented!(),
            Decl::Func(func) => {
                let clauses = (func.clauses.iter())
                    // Our elims should be enough
                    .filter(|clause| elims.len() >= clause.patterns.len())
                    // Should not be an absurd clause
                    .filter(|clause| !clause.is_absurd())
                    .cloned()
                    .collect();
                unfold_func(id, clauses, tcs, elims)
            }
            Decl::ClausePlaceholder => unreachable!(),
        },
    }
}

/// Build up a substitution and unfold the declaration.
fn unfold_func(f: Ident, clauses: Vec<Clause>, tcs: TCS, elims: Vec<Elim>) -> ValTCM {
    for clause in clauses {
        unimplemented!()
    }
    Err(TCE::CantFindPattern(f))
}

fn elims_to_terms(elims: Vec<Elim>) -> TCM<Vec<Term>> {
    elims
        .into_iter()
        .map(Elim::try_into_app)
        .collect::<Result<_, String>>()
        .map_err(TCE::NotTerm)
}
