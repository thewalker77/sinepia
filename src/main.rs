#[macro_use]
extern crate lalrpop_util;

use sinepia_lambda::{ast::*, Context, Error as SError, SubstTracker};
use rustyline::error::ReadlineError;
use rustyline::Editor;

lalrpop_mod!(#[allow(clippy::all)] pub parser);
use parser::StmtParser;
fn main() {
    let mut ctx = Context::new();
    let mut trk = SubstTracker::new();
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                process(line, &mut ctx, &mut trk);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
fn process(line: String, ctx: &mut Context, trk: &mut SubstTracker) {
    let stmt = match StmtParser::new().parse(&line) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{:#}", e);
            return;
        }
    };
    match stmt {
        Statement::Axiom(a) => process_axiom(a, ctx, trk),
        Statement::TH(ut) => process_unproven(ut, ctx, trk),
        Statement::PF(pt) => process_proven(pt, ctx, trk),
    }
}

fn process_axiom(ax: Axiom, ctx: &mut Context, trk: &mut SubstTracker) {
    if let Err(e) = ctx.add_axiom(ax.name, ax.typ, trk) {
        display(e);
    }
}

fn process_unproven(th: Theorem, ctx: &mut Context, trk: &mut SubstTracker) {
    if let Err(e) = ctx.add_theorem(th.name, th.typ, trk) {
        display(e);
    }
}

fn process_proven(pf: Proof, ctx: &mut Context, trk: &mut SubstTracker) {
    if let Err(e) = ctx.extend_type(&pf.name, pf.value, trk) {
        display(e);
    }
}

fn display(e: SError) {
    match e {
        SError::CannotProveAxiom => println!("Axioms cannot be proven"),
        SError::VariableNotFound => println!("Bound variable is not found"),
        SError::ExprDoesNotTypeCheck(_) => println!("Expression does not type check"),
        SError::TypesDoesNotMatch(_) => println!("Types do not match"),
        SError::AlreadyExists => println!("Axiom or theorem with the same name already exists"),
    }
}
