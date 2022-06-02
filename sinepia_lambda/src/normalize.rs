use crate::{ast::*, Context, Subst, SubstTracker};
pub trait Normalize {
    fn normalize(&self, ctx: &Context, trk: &mut SubstTracker) -> Option<Expr>;
}

impl Normalize for Variable {
    fn normalize(&self, ctx: &Context, trk: &mut SubstTracker) -> Option<Expr> {
        if ctx.contains(self) {
            match ctx.lookup_value(self) {
                Some(e) => e.normalize(ctx, trk),
                None => Some(Expr::Var(self.clone())),
            }
        } else {
            None
        }
    }
}

impl Normalize for Universe {
    fn normalize(&self, _: &Context, _: &mut SubstTracker) -> Option<Expr> {
        Some(Expr::Uni(self.clone()))
    }
}

impl Normalize for Application {
    fn normalize(&self, ctx: &Context, trk: &mut SubstTracker) -> Option<Expr> {
        let e2 = self.e2.normalize(ctx, trk)?;
        match self.e1.normalize(ctx, trk)? {
            Expr::Lambda(l) => {
                let mut e1 = l.e.clone();
                e1.subst(&l.x, &e2, trk);
                e1.normalize(ctx, trk)
            },
            //unreachable ?
            e1 => Some(Expr::App(Application {
                e1: Box::new(e1),
                e2: Box::new(e2),
            })),
        }
    }
}

impl<T> Normalize for Abstraction<T>
where
    Abstraction<T>: Into<Expr>,
{
    fn normalize(&self, ctx: &Context, trk: &mut SubstTracker) -> Option<Expr> {
        let t = self.t.normalize(ctx, trk)?;
        let ctx2 = ctx.with_type(self.x.clone(), t.clone());
        let e = self.e.normalize(&ctx2, trk)?;
        Some(
            Abstraction::<T> {
                x: self.x.clone(),
                t: Box::new(t),
                e: Box::new(e),
                _ty: self._ty,
            }
            .into(),
        )
    }
}
impl Normalize for Expr {
    fn normalize(&self, ctx: &Context, trk: &mut SubstTracker) -> Option<Expr> {
        match self {
            Expr::Var(v) => v.normalize(ctx, trk),
            Expr::Uni(u) => u.normalize(ctx, trk),
            Expr::Pi(p) => p.normalize(ctx, trk),
            Expr::Lambda(l) => l.normalize(ctx, trk),
            Expr::App(a) => a.normalize(ctx, trk),
        }
    }
}
