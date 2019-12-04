use std::collections::HashMap;
use std::ops::Add;

use voile_util::tags::Plicit;
use voile_util::uid::{DBI, UID};

use crate::check::monad::{TCMS, TCS};
use crate::syntax::abs::AbsCopat;
use crate::syntax::core::{Bind, Term};
use crate::syntax::pat::{Copat, Pat, PatCommon};

use super::super::term::is_eta_var_borrow;

/// A user pattern and a core term that they should equal
/// after splitting is complete.
/// [Agda](https://hackage.haskell.org/package/Agda-2.6.0.1/docs/src/Agda.Syntax.Abstract.html#ProblemEq).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Equation {
    /// The pattern causes this problem.
    pub in_pat: AbsCopat,
    pub inst: Term,
    pub ty: Term,
}

impl PatCommon for Equation {
    fn is_split(&self) -> bool {
        self.in_pat.is_split()
    }
}

/// [Agda](https://hackage.haskell.org/package/Agda-2.6.0.1/docs/src/Agda.TypeChecking.Rules.LHS.Problem.html#AsBinding).
#[derive(Debug, Clone)]
pub struct AsBind {
    pub name: UID,
    pub term: Term,
    pub ty: Term,
}

impl From<AsBind> for Bind {
    fn from(asb: AsBind) -> Self {
        Bind::new(Plicit::Ex, asb.name, asb.ty, Some(asb.term))
    }
}

impl AsBind {
    pub fn new(name: UID, term: Term, ty: Term) -> Self {
        Self { name, term, ty }
    }
}

/// Classified patterns, called `LeftoverPatterns` in Agda.
/// [Agda](https://hackage.haskell.org/package/Agda-2.6.0.1/docs/src/Agda.TypeChecking.Rules.LHS.html#LeftoverPatterns).
#[derive(Debug, Clone)]
pub struct PatClass {
    /// Number of absurd patterns.
    pub absurd_count: usize,
    pub as_binds: Vec<AsBind>,
    pub other_pats: Vec<AbsCopat>,
    /// Supposed to be an `IntMap`.
    pub pat_vars: PatVars,
}

impl Add for PatClass {
    type Output = Self;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        self.other_pats.append(&mut rhs.other_pats);
        self.as_binds.append(&mut rhs.as_binds);
        for (dbi, mut names) in rhs.pat_vars.into_iter() {
            let mut existing = self.pat_vars.remove(&dbi).unwrap_or_default();
            existing.append(&mut names);
            self.pat_vars.insert(dbi, existing);
        }
        Self {
            absurd_count: self.absurd_count + rhs.absurd_count,
            ..self
        }
    }
}

pub type PatVars = HashMap<DBI, Vec<UID>>;

pub fn classify_eqs(mut tcs: TCS, eqs: Vec<Equation>) -> TCMS<PatClass> {
    let mut pat_vars = PatVars::new();
    let mut other_pats = Vec::with_capacity(eqs.len());
    let mut as_binds = Vec::with_capacity(eqs.len());
    let mut absurd_count = 0usize;
    for eq in eqs {
        match eq.in_pat {
            Copat::App(Pat::Absurd) => absurd_count += 1,
            Copat::App(Pat::Var(x)) => {
                let (i, new_tcs) = is_eta_var_borrow(tcs, &eq.inst, &eq.ty)?;
                tcs = new_tcs;
                if let Some(i) = i {
                    pat_vars
                        .entry(i)
                        .and_modify(|v| v.push(x))
                        .or_insert(vec![x]);
                } else {
                    as_binds.push(AsBind::new(x, eq.inst, eq.ty));
                }
            }
            p => other_pats.push(p),
        }
    }
    let class = PatClass {
        absurd_count,
        other_pats,
        pat_vars,
        as_binds,
    };
    Ok((class, tcs))
}
