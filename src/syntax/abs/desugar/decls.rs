use voile_util::loc::{Ident, Labelled, ToLoc};
use voile_util::tags::Plicit;
use voile_util::uid::{next_uid, GI};

use crate::syntax::abs::{Abs, AbsDecl, AbsPat, AbsTele, Bind};
use crate::syntax::pat::{Copat, Pat};
use crate::syntax::surf::{Expr, ExprCopat, ExprDecl, ExprPat, NamedTele};

use super::{desugar_expr, DesugarErr, DesugarM, DesugarState};

pub fn desugar_decls(state: DesugarState, decls: Vec<ExprDecl>) -> DesugarM {
    decls.into_iter().try_fold(state, desugar_decl)
}

/// Note: this function will not clear the local scope.
pub fn desugar_telescope(
    mut state: DesugarState,
    signature: NamedTele,
) -> DesugarM<(Ident, AbsTele, DesugarState)> {
    let ident = signature.name;
    // The capacity is really guessed. Who knows?
    let mut tele = AbsTele::with_capacity(signature.tele.len() + 2);
    for mut param in signature.tele {
        let (ty, new_state) = desugar_expr(state, param.ty)?;
        state = new_state;
        let mut intros = |name: Ident, licit: Plicit, ty: Abs| {
            let uid = unsafe { next_uid() };
            state.local.insert(name.text, uid);
            tele.push(Bind::new(licit, uid, ty));
        };
        match param.names.len() {
            0 => tele.push(Bind::new(param.licit, unsafe { next_uid() }, ty)),
            1 => intros(param.names.remove(0), param.licit, ty),
            _ => {
                for name in param.names {
                    intros(name, param.licit, ty.clone())
                }
            }
        }
    }
    Ok((ident, tele, state))
}

pub fn desugar_pattern(state: DesugarState, pat: ExprPat) -> DesugarM<(AbsPat, DesugarState)> {
    match pat {
        Pat::Var(name) => {
            let mut st = state;
            let uid = unsafe { next_uid() };
            st.local.insert(name.text, uid);
            Ok((Pat::Var(uid), st))
        }
        // The `head` is pseudo (see `surf::parse`), only `head.name` is real.
        Pat::Cons(is_forced, mut head, params) => {
            let (head_ix, cons) = state
                .lookup_by_name(&head.name.text)
                .ok_or_else(|| DesugarErr::UnresolvedReference(head.name.clone()))?;
            unimplemented!()
        }
        Pat::Forced(term) => {
            let (abs, st) = desugar_expr(state, term)?;
            Ok((Pat::Forced(abs), st))
        }
        Pat::Refl => Ok((Pat::Refl, state)),
        Pat::Absurd => Ok((Pat::Absurd, state)),
    }
}

pub fn desugar_clause(
    mut state: DesugarState,
    defn_ix: GI,
    name: Ident,
    pats: Vec<ExprCopat>,
    body: Expr,
) -> DesugarM {
    let mut abs_pats = Vec::with_capacity(pats.len());
    for copat in pats {
        let pat = match copat {
            Copat::App(app) => {
                let (pat, st) = desugar_pattern(state, app)?;
                state = st;
                Copat::App(pat)
            }
            Copat::Proj(s) => Copat::Proj(s),
        };
        abs_pats.push(pat);
    }
    Ok(state)
}

pub fn desugar_decl(state: DesugarState, decl: ExprDecl) -> DesugarM {
    use ExprDecl::*;
    match decl {
        Defn(name, sig) => {
            let (sig, mut state) = desugar_expr(state, sig)?;
            state.ensure_local_emptiness();
            let abs_decl = AbsDecl::defn(name.loc + sig.loc(), name, sig);
            state.decls.push(abs_decl);
            Ok(state)
        }
        Cls(name, pats, body) => match state.lookup_by_name(&name.text) {
            Some((ix, AbsDecl::Defn { .. })) => desugar_clause(state, ix, name, pats, body),
            None => {
                let mut state = state;
                let meta = Abs::Meta(name.clone(), state.fresh_meta());
                let decl_len = state.decl_len();
                let mut state = desugar_clause(state, decl_len, name.clone(), pats, body)?;
                state.ensure_local_emptiness();
                let defn = AbsDecl::defn(name.loc, name, meta);
                state.decls.push(defn);
                Ok(state)
            }
            Some((_, other)) => Err(DesugarErr::NotDefn(other.decl_name().clone())),
        },
        Data(signature, conses) => {
            let (name, tele, mut state) = desugar_telescope(state, signature)?;
            state.decls.reserve(conses.len());
            unimplemented!()
        }
        Codata(signature, fields) => {
            let (name, tele, mut state) = desugar_telescope(state, signature)?;
            state.decls.reserve(fields.len());
            unimplemented!()
        }
    }
}
