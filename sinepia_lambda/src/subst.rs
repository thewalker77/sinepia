use crate::ast::*;
use crate::Uinf;

/// Abstract data type that is passed around to every operation that will require
/// [Subst]. The goal of this structure is to help with generating new `id` for
/// [GenSym].
pub struct SubstTracker {
    current_level: Uinf,
}
impl Default for SubstTracker {
    fn default() -> Self {
        Self::new()
    }
}
impl SubstTracker {
    pub fn new() -> Self {
        SubstTracker { current_level: 0 }
    }
    fn refresh_var(&mut self, var: &Variable) -> Variable {
        let name = match var {
            Variable::GenSym(s) => s.name.to_owned(),
            Variable::Str(s) => s.name.to_owned(),
        };
        self.current_level += 1;
        let id = self.current_level;
        Variable::GenSym(GenSym { name, id })
    }
}

/// This trait defines how can expressions be substituted for variables. in a given expression.
pub trait Subst {
    /// replaces [Variable] `v` with [Expr] `e` in `self`
    fn subst(&mut self, v: &Variable, e: &Expr, ctx: &mut SubstTracker);
}

impl Subst for Expr {
    fn subst(&mut self, v: &Variable, e: &Expr, ctx: &mut SubstTracker) {
        match self {
            Expr::Var(v2) => {
                if v == v2 {
                    *self = e.clone()
                }
            }
            Expr::Pi(abs) => abs.subst(v, e, ctx),
            Expr::Lambda(abs) => abs.subst(v, e, ctx),
            Expr::App(app) => app.subst(v, e, ctx),
            Expr::Uni(_) => (),
        }
    }
}

impl Subst for Application {
    fn subst(&mut self, v: &Variable, e: &Expr, ctx: &mut SubstTracker) {
        self.e1.subst(v, e, ctx);
        self.e2.subst(v, e, ctx);
    }
}

impl<T> Subst for Abstraction<T> {
    fn subst(&mut self, v: &Variable, e: &Expr, ctx: &mut SubstTracker) {
        let x = ctx.refresh_var(&self.x);
        self.t.subst(v, e, ctx);
        self.e.subst(&self.x, &Expr::Var(x.clone()), ctx);
        self.e.subst(v, e, ctx);
        self.x = x;
    }
}
