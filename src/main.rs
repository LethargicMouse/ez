use std::{env::args, process::ExitCode};

mod ast;
mod check;
mod cli;
mod codegen;
mod fail;
mod hir;
mod lex;
mod lir;
mod mono;
mod parse;
mod util;

#[cfg(test)]
mod tests;

fn main() -> ExitCode {
    match cli::run(args()) {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
