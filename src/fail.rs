use std::{
    fmt::Display,
    io,
    path::{Path, PathBuf},
};

use crate::{
    ast::Source,
    lex::{Pos, Tok},
};

macro_rules! error {
    ($fmt:expr $(, $($arg:expr),+)? $(,)?) => {{
        eprintln!(concat!("\x1b[31;1merror:\x1b[0m ", $fmt) $(, $($arg),+)?);
        Fail
    }};
}

macro_rules! error_loc {
    ($source:expr, $pos:expr, $fmt:expr $(, $($arg:expr),+)? $(,)?) => {
        error!(concat!("{}:{}: ", $fmt, "\n    {}\n    {}\n"), $source.path.display(), $pos $(, $($arg),+)?, $source.lines[$pos.line as usize - 1], SymbolMark($pos.symbol))
    };
}

#[derive(Debug)]
pub struct Fail;

pub fn no_path(cmd: String) -> Fail {
    error!("expected path for `{}`", cmd)
}

pub fn bad_command(command: String) -> Fail {
    error!("unknown command: `{}`", command)
}

pub fn run_cmd(program: &Path, err: io::Error) -> Fail {
    error!("failed to run `{}`: {}", program.display(), err)
}

pub fn file_read(path: &Path, err: io::Error) -> Fail {
    error!("failed to read `{}`: {}", path.display(), err)
}

pub fn fs_create(path: &Path, err: io::Error) -> Fail {
    error!("failed to create `{}`: {}", path.display(), err)
}

pub fn file_write(path: &Path, err: io::Error) -> Fail {
    error!("failed to write to `{}`: {}", path.display(), err)
}

pub struct Errors {
    count: u32,
}

impl Errors {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn no_main(&mut self, path: &Path) -> Fail {
        self.count += 1;
        error!("{}: `main` function not declared\n", path.display())
    }

    pub fn parse(&mut self, source: &Source, pos: Pos, expected: &str, found: Tok) -> Fail {
        self.count += 1;
        error_loc!(
            source,
            pos,
            "expected {}, found {}",
            expected,
            found.as_str(),
        )
    }

    pub fn cannot_infer(&mut self, source: &Source, pos: Pos) {
        self.count += 1;
        error_loc!(source, pos, "cannot infer type");
    }

    pub fn present(&self) -> bool {
        self.count != 0
    }

    pub fn not_declared(&mut self, source: &Source, loc: Pos, name: &str) {
        self.count += 1;
        error_loc!(source, loc, "item `{}` is not declared", name);
    }
}

impl Drop for Errors {
    fn drop(&mut self) {
        if self.count != 0 {
            error!("compilation failed with {} errors", self.count);
        }
    }
}

struct SymbolMark(u32);

impl Display for SymbolMark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 1..self.0 {
            write!(f, " ")?;
        }
        write!(f, "^")
    }
}
