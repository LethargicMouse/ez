use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

use crate::{
    fail::{self, Fail},
    lir::*,
};

pub fn run(program: Program, path: &Path) -> Result<(), Fail> {
    let mut file = File::create(path).map_err(|err| fail::fs_create(path, err))?;
    program
        .dump(&mut file)
        .map_err(|err| fail::file_write(path, err))
}

impl Program {
    fn dump(self, file: &mut File) -> io::Result<()> {
        write!(file, "target triple = \"x86_64-pc-linux-gnu\"")?;
        write!(file, "\ndefine i32 @main() {{")?;
        write!(file, "\n  %ec = call i32 @f0()")?;
        write!(file, "\n  ret i32 %ec")?;
        write!(file, "\n}}")?;
        for (i, fun) in self.funs.into_iter().enumerate() {
            fun.dump(i, file)?;
        }
        Ok(())
    }
}

impl Fun {
    fn dump(self, id: usize, file: &mut File) -> io::Result<()> {
        write!(file, "\ndefine {} @f{}() {{", self.ret_typ, id)?;
        for block in self.blocks {
            block.dump(file)?;
        }
        write!(file, "\n}}")?;
        Ok(())
    }
}

impl Block {
    fn dump(self, file: &mut File) -> io::Result<()> {
        self.out.dump(file)
    }
}

impl Out {
    fn dump(self, file: &mut File) -> io::Result<()> {
        match self {
            Out::Ret(expr) => write!(file, "\n  ret {}", expr),
        }
    }
}
