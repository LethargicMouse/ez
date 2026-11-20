use std::{path::Path, process::Command};

use crate::{
    cli::{compile, run_file},
    fail::{self, Fail},
};

#[test]
fn empty() -> Result<(), Fail> {
    let code = run_file("tests/Empty.ez".into())?;
    assert_eq!(123, code);
    Ok(())
}

#[test]
fn calc() -> Result<(), Fail> {
    compile("tests/Call.ez".into())?;
    let out = Path::new("build/out");
    let out = Command::new(out)
        .output()
        .map_err(|err| fail::run_cmd(out, err))?;
    assert_eq!(Some(0), out.status.code());
    assert_eq!(b"123", out.stdout.as_slice());
    assert_eq!(b"", out.stderr.as_slice());
    Ok(())
}
