use crate::{ast::*, Error, Normalize, Subst, SubstTracker, TypeInfer};
use std::{collections::HashMap, mem};

#[derive(Clone)]
struct AxiomEntry {
    typ: Expr,
}

#[derive(Clone)]
struct ProvedTheoremEntry {
    typ: Expr,
    val: Expr,
}

#[derive(Clone)]
struct UnprovedTheoremEntry {
    typ: Expr,
}

#[derive(Clone)]
enum Entry {
    Axiom(AxiomEntry),
    Proved(ProvedTheoremEntry),
    Unproved(UnprovedTheoremEntry),
}

/// The `Γ` in `Γ |- x: A`
#[derive(Clone)]
pub struct Context {
    ctx: HashMap<Variable, Entry>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Returns a new context
    pub fn new() -> Self {
        Context {
            ctx: Default::default(),
        }
    }
    /// Returns [true] if there is an axiom or theorem with
    /// name `v` and [false] otherwise.
    pub fn contains(&self, v: &Variable) -> bool {
        self.ctx.contains_key(v)
    }
    /// returns [true] if `v` is refers to an axiom, and
    /// [false] otherwise.
    pub fn is_axiom(&self, v: &Variable) -> bool {
        matches!(self.ctx.get(v), Some(Entry::Axiom(_)))
    }
    /// returns [true] if `v` refers to a proven theory, and
    /// [false] otherwise.
    pub fn is_proven_theory(&self, v: &Variable) -> bool {
        matches!(self.ctx.get(v), Some(Entry::Proved(_)))
    }
    /// returns [true] if `v` refers to a unproven theory and
    /// [false] otherwise.
    pub fn is_unproven_theory(&self, v: &Variable) -> bool {
        matches!(self.ctx.get(v), Some(Entry::Unproved(_)))
    }
    /// returns the value stored for `v` assuming it is a proven theory.
    pub fn lookup_value(&self, v: &Variable) -> Option<&Expr> {
        self.ctx.get(v).and_then(|e: &Entry| {
            if let Entry::Proved(p) = e {
                Some(&p.val)
            } else {
                None
            }
        })
    }
    /// Returns the type of `v` whether it is axiom, proven theory
    /// or unproven theory as long as it exists in the context.
    pub fn lookup_type(&self, v: &Variable) -> Option<&Expr> {
        self.ctx.get(v).map(|x: &Entry| match x {
            Entry::Axiom(a) => &a.typ,
            Entry::Proved(t) => &t.typ,
            Entry::Unproved(u) => &u.typ,
        })
    }
    /// Stores a new value for the unproven theorem represented by `v`.
    /// Returns the old value if it exists, otherwise [None] is returned.
    /// Error triggering:
    /// 1- `v` represents an axiom.
    /// 2- `v` does not exist.
    /// 2- `expr` does not type check
    /// 3- type of `exp` and type of `v` does not match.
    pub fn extend_type(
        &mut self,
        v: &Variable,
        expr: Expr,
        trk: &mut SubstTracker,
    ) -> Result<Option<Expr>, Error> {
        if self.is_axiom(v) {
            return Err(Error::CannotProveAxiom);
        }
        let typ = match self.lookup_type(v) {
            Some(typ) => typ,
            None => return Err(Error::VariableNotFound),
        };
        let typ2 = match expr.type_infer(self.clone(), trk) {
            Some(typ2) => typ2,
            None => return Err(Error::ExprDoesNotTypeCheck(None)),
        };
        if self.types_equal(trk, typ, &typ2) {
            Ok(self.extend_type_unchecked(v, expr))
        } else {
            Err(Error::TypesDoesNotMatch((typ.clone(), typ2)))
        }
    }
    /// Inserts element into the context without checking anything.
    /// Assumed checks:
    ///     1- v is not an axiom
    ///     2- expr type checks
    /// Returns the old proof of the existing theorem if it exists, or returns none otherwise.
    fn extend_type_unchecked(&mut self, v: &Variable, mut expr: Expr) -> Option<Expr> {
        let entry = self.ctx.get_mut(v).unwrap();
        match entry {
            Entry::Axiom(_) => unreachable!(),
            Entry::Proved(p) => {
                mem::swap(&mut p.val, &mut expr);
                Some(expr)
            }
            Entry::Unproved(u) => {
                let p = ProvedTheoremEntry {
                    val: expr,
                    typ: u.typ.clone(),
                };
                *entry = Entry::Proved(p);
                None
            }
        }
    }
    /// Adds a new theorem without its proof. Returns error
    /// if a theorem or an axiom with the same name exists
    /// or if the theorem does not type check under the
    /// current [Context]
    pub fn add_theorem(
        &mut self,
        v: Variable,
        typ: Expr,
        trk: &mut SubstTracker,
    ) -> Result<(), Error> {
        if self.contains(&v) {
            return Err(Error::AlreadyExists);
        }
        if typ.type_infer(self.clone(), trk).is_none() {
            return Err(Error::ExprDoesNotTypeCheck(None));
        }
        self.ctx
            .insert(v, Entry::Unproved(UnprovedTheoremEntry { typ }));
        Ok(())
    }
    /// Adds a new axiom. Returns error if a theorem or an axiom with
    /// the same name exists or if the axiom does not type check under
    /// the current [Context]
    pub fn add_axiom(
        &mut self,
        v: Variable,
        typ: Expr,
        trk: &mut SubstTracker,
    ) -> Result<(), Error> {
        if self.contains(&v) {
            return Err(Error::AlreadyExists);
        }
        if typ.type_infer(self.clone(), trk).is_none() {
            return Err(Error::ExprDoesNotTypeCheck(None));
        }
        self.ctx.insert(v, Entry::Axiom(AxiomEntry { typ }));
        Ok(())
    }
    /// Returns a new context with unproven theory with name
    /// [v] and body [ty].
    pub(crate) fn with_type(&self, v: Variable, typ: Expr) -> Self {
        let mut ctx = self.clone();
        ctx.ctx
            .insert(v, Entry::Unproved(UnprovedTheoremEntry { typ }));
        ctx
    }
    pub fn types_equal(&self, trk: &mut SubstTracker, e1: &Expr, e2: &Expr) -> bool {
        let e1 = match e1.normalize(self, trk) {
            Some(e) => e,
            None => return false,
        };
        let e2 = match e2.normalize(self, trk) {
            Some(e) => e,
            None => return false,
        };
        normalized_types_equal(&e1, &e2, trk)
    }
}

fn normalized_types_equal(e1: &Expr, e2: &Expr, trk: &mut SubstTracker) -> bool {
    match (e1, e2) {
        (Expr::Var(v1), Expr::Var(v2)) => v1 == v2,
        (
            Expr::App(Application { e1: e11, e2: e12 }),
            Expr::App(Application { e1: e21, e2: e22 }),
        ) => normalized_types_equal(e11, e21, trk) && normalized_types_equal(e12, e22, trk),
        (Expr::Uni(u1), Expr::Uni(u2)) => u1 == u2,
        (Expr::Lambda(l1), Expr::Lambda(l2)) => normalized_abs_equal(l1, l2, trk),
        (Expr::Pi(p1), Expr::Pi(p2)) => normalized_abs_equal(p1, p2, trk),
        (_, _) => false,
    }
}

fn normalized_abs_equal<T>(
    a1: &Abstraction<T>,
    a2: &Abstraction<T>,
    trk: &mut SubstTracker,
) -> bool {
    if !normalized_types_equal(&a1.t, &a2.t, trk) {
        return false;
    }
    let mut e2 = a2.e.clone();
    let xvar = Expr::Var(a1.x.clone());
    e2.subst(&a2.x, &xvar, trk);
    normalized_types_equal(&a1.e, &e2, trk)
}
