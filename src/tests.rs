use crate::cli::run_file;

#[test]
fn empty() {
    let code = run_file("tests/Empty.ez".into()).unwrap_or(1);
    assert_eq!(123, code);
}
