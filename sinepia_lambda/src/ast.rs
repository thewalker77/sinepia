use super::Uinf;
use std::marker::PhantomData;

/// Top level statement. Usually a module will be a collection of those statements.
#[derive(Debug)]
pub enum Statement {
    /// Axiom is a named type that will not be proven.
    Axiom(Axiom),
    /// A theorem.
    TH(Theorem),
    /// Proof.
    PF(Proof),
}

#[derive(Debug)]
pub struct Axiom {
    /// The name of the Axiom.
    pub name: Variable,
    /// The data type representing that Axiom
    pub typ: Expr,
}

#[derive(Debug)]
pub struct Theorem {
    /// The name of the theorem.
    pub name: Variable,
    /// The data type representing the body of the theorem.
    pub typ: Expr,
}

#[derive(Debug)]
pub struct Proof {
    /// The name of the theorem.
    pub name: Variable,
    /// The Theorem's proof.
    pub value: Expr,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Var(Variable),
    Uni(Universe),
    Pi(Pi),
    Lambda(Lambda),
    App(Application),
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum Variable {
    /// Users will always want to use this variant of [Variable].
    Str(StrVar),
    /// This variable is automatically generated during type analysis and never exposed to end user.
    GenSym(GenSym),
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct StrVar {
    pub name: String,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct GenSym {
    pub(crate) name: String,
    pub(crate) id: Uinf,
}

/// Representation of type universes.
///
/// we are only limited to
/// 340282366920938463463374607431768211455 nested universe :(
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Universe {
    pub level: Uinf,
}

/// Function Application
#[derive(Clone, Debug)]
pub struct Application {
    pub e1: Box<Expr>,
    pub e2: Box<Expr>,
}

/// Abstraction
///
/// Depending on `T` this maybe a [Lambda] expression of [Pi] expresion.
#[derive(Clone, Debug)]
pub struct Abstraction<T> {
    /// Bound variable
    pub x: Variable,
    /// Type of the bound variable
    pub t: Box<Expr>,
    /// Expression containing the bound variable
    pub e: Box<Expr>,
    /// Phantom type: This is to differentiate between Pi and lambdas
    pub _ty: PhantomData<T>,
}

#[derive(Clone, Debug)]
pub struct PiPhantom;

#[derive(Clone, Debug)]
pub struct LambdaPhantom;

/// Pi types
pub type Pi = Abstraction<PiPhantom>;
impl From<Pi> for Expr {
    fn from(l: Pi) -> Self {
        Expr::Pi(l)
    }
}
/// Lambda types
pub type Lambda = Abstraction<LambdaPhantom>;
impl From<Lambda> for Expr {
    fn from(l: Lambda) -> Self {
        Expr::Lambda(l)
    }
}
