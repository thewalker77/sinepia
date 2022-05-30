pub mod ast;
mod context;
mod normalize;
mod subst;
mod typecheck;
pub use context::Context;
pub use normalize::Normalize;
pub use subst::{Subst, SubstTracker};
pub use typecheck::TypeInfer;

use ast::Expr;
/// In theory this is arbitrarily long integer.
/// In practice we don't need integer that is that long.
/// u128 should be sufficient for now.
pub type Uinf = u128;
pub type AdditionalErr = Option<Box<Error>>;

pub enum Error {
    /// This error is returned when user tries to extend an axiom
    /// (usually via [Context::extend_type]) with proof.
    CannotProveAxiom,
    /// This error is returned via [Context::extend_type] when we try to
    /// prove a theorem who cannot be identified by its name (does not exist)
    VariableNotFound,
    /// Returned when an expresssion provided as proof by [Context::extend_type]
    /// fails to typecheck
    ExprDoesNotTypeCheck(AdditionalErr),
    TypesDoesNotMatch((Expr, Expr)),
    AlreadyExists,
}
