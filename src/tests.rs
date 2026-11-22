use std::{path::PathBuf, process::Command};

use crate::{
    cli::compile,
    fail::{self, Fail},
};

fn test_file(name: &str, code: i32, stdout: &str, stderr: &str) -> Result<(), Fail> {
    let path = PathBuf::from(format!("tests/{}.ez", name));
    let out_ll_path = PathBuf::from(format!("build/{}.ll", name));
    let out_path = PathBuf::from(format!("build/{}", name));
    compile(path, &out_ll_path, &out_path)?;
    let out = Command::new(&out_path)
        .output()
        .map_err(|err| fail::run_cmd(&out_path, err))?;
    assert_eq!(Some(code), out.status.code());
    assert_eq!(stdout.as_bytes(), out.stdout.as_slice());
    assert_eq!(stderr.as_bytes(), out.stderr.as_slice());
    Ok(())
}

#[test]
fn empty() -> Result<(), Fail> {
    test_file("Empty", 123, "", "")
}

#[test]
fn calc() -> Result<(), Fail> {
    test_file("Call", 0, "123", "")
}
