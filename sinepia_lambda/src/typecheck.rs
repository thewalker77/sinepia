use crate::subst::{Subst, SubstTracker};
use crate::{ast::*, normalize::*, Context};
use std::{cmp::max, marker::PhantomData};
/// A data type that derives this trait can be type inferred.
pub trait TypeInfer {
    fn type_infer(&self, ctx: Context, trk: &mut SubstTracker) -> Option<Expr>;
}

///```text
///           Γ(x) = A
///         -------------
///           Γ |- x: A
///```
impl TypeInfer for Variable {
    fn type_infer(&self, ctx: Context, _: &mut SubstTracker) -> Option<Expr> {
        ctx.lookup_type(self).cloned()
    }
}
///```text
///         ----------------
///           Γ |- Uᵢ: Uᵢ₊₁
///```
/// Note that if `Γ |- Uᵢ: Uᵢ₊₁` and `Γ |- Uᵢ: Uᵢ₊₁` then `Γ |- Uᵢ: Uᵢ₊₂` But we will only
/// generate the smallest suitable type. This has some subtle implication. One example is in
/// typing [Pi].
impl TypeInfer for Universe {
    fn type_infer(&self, _: Context, _: &mut SubstTracker) -> Option<Expr> {
        Some(Expr::Uni(Universe {
            level: self.level + 1,
        }))
    }
}
///```text
///            Γ |- A: Uᵢ     Γ, x: A |- y: Uᵢ
///         ------------------------------------
///                 Γ |- (Π x: A, y): Uᵢ
///```
/// Note that the implementation is is slightly different where we type
/// `A: Uᵢ`, `Y: Uⱼ` and we pick the max, that is because if A: Uᵢ, then A: Uᵢ₊₁

impl TypeInfer for Pi {
    fn type_infer(&self, ctx: Context, trk: &mut SubstTracker) -> Option<Expr> {
        let k1 = infer_universe(ctx.clone(), &self.t, trk)?;
        let ctx2 = ctx.with_type(self.x.clone(), self.t.as_ref().clone());
        let k2 = infer_universe(ctx2, &self.e, trk)?;
        Some(Expr::Uni(max(k1, k2)))
    }
}
///````text
///         Γ |- A: Uᵢ      Γ, x: A |- y: B
///     ---------------------------------------
///         Γ |- (λ x : A, y) : (Π x: A, B)
///```
impl TypeInfer for Lambda {
    fn type_infer(&self, ctx: Context, trk: &mut SubstTracker) -> Option<Expr> {
        infer_universe(ctx.clone(), &self.t, trk)?;
        let ctx2 = ctx.with_type(self.x.clone(), self.t.as_ref().clone());
        let te = self.e.type_infer(ctx2, trk)?;
        Some(Expr::Pi(Pi {
            x: self.x.clone(),
            t: self.t.clone(),
            e: Box::new(te),
            _ty: PhantomData,
        }))
    }
}

///```text
///             Γ |- m : (Π x: A, Y)       Γ |- n : A
///         ----------------------------------------------
///                       Γ |- m n : Y[n/x]
///```
impl TypeInfer for Application {
    fn type_infer(&self, ctx: Context, trk: &mut SubstTracker) -> Option<Expr> {
        let abs = infer_pi(ctx.clone(), &self.e1, trk)?;
        let te = self.e2.type_infer(ctx.clone(), trk)?;
        if ctx.types_equal(trk, &abs.t, &te) {
            let mut ret = abs.e.as_ref().clone();
            ret.subst(&abs.x, &self.e2, trk);
            Some(ret)
        } else {
            None
        }
    }
}

/// This is thin abstraction layer, no logic here.
impl TypeInfer for Expr {
    fn type_infer(&self, ctx: Context, trk: &mut SubstTracker) -> Option<Expr> {
        match self {
            Expr::Var(v) => v.type_infer(ctx, trk),
            Expr::Uni(u) => u.type_infer(ctx, trk),
            Expr::Pi(p) => p.type_infer(ctx, trk),
            Expr::Lambda(l) => l.type_infer(ctx, trk),
            Expr::App(a) => a.type_infer(ctx, trk),
        }
    }
}

fn infer_universe(ctx: Context, e: &Expr, trk: &mut SubstTracker) -> Option<Universe> {
    let u = e.type_infer(ctx.clone(), trk)?;
    if let Expr::Uni(normalized) = u.normalize(&ctx, trk)? {
        Some(normalized)
    } else {
        None
    }
}
fn infer_pi(ctx: Context, e: &Expr, trk: &mut SubstTracker) -> Option<Pi> {
    let p = e.type_infer(ctx.clone(), trk)?;
    if let Expr::Pi(normalized) = p.normalize(&ctx, trk)? {
        Some(normalized)
    } else {
        None
    }
}
