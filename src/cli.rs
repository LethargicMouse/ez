use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::{self, exit},
};

use crate::{
    ast::Module,
    check, codegen,
    fail::{self, Fail},
    mono,
};

pub fn run(args: impl Iterator<Item = String>) -> Result<(), Fail> {
    let args = Args::parse(args)?;
    match args.command {
        Command::Run(path) => exit(run_file(path)?),
        Command::Help => show_help(&args.exe_name),
    }
    Ok(())
}

fn show_help(exe_name: &str) {
    eprintln!("usage: {} <command>", exe_name);
    eprintln!("commands:");
    eprintln!("  help");
    eprintln!("  run <path>");
}

pub fn run_file(path: PathBuf) -> Result<i32, Fail> {
    let out_path = Path::new("build/out");
    compile(path)?;
    Ok(process::Command::new(out_path)
        .status()
        .map_err(|err| fail::run_cmd(out_path, err))?
        .code()
        .unwrap_or(1))
}

fn compile(path: PathBuf) -> Result<(), Fail> {
    let module = Module::read(path)?;
    let program = check::run(module)?;
    let program = mono::run(program);
    let build_dir_path = Path::new("build");
    create_dir_all(build_dir_path).map_err(|err| fail::fs_create(build_dir_path, err))?;
    let out_ll_path = Path::new("build/out.ll");
    codegen::run(program, out_ll_path)?;
    let clang = Path::new("clang");
    let clang_code = process::Command::new(clang)
        .arg("-o")
        .arg("build/out")
        .arg(out_ll_path)
        .status()
        .map_err(|err| fail::run_cmd(clang, err))?
        .code()
        .unwrap_or(1);
    assert!(clang_code == 0);
    Ok(())
}

struct Args {
    exe_name: String,
    command: Command,
}

impl Args {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self, Fail> {
        let exe_name = args.next().expect("should have exe name as arg 0");
        Ok(Self {
            command: Command::parse(&mut args).inspect_err(|_| show_help(&exe_name))?,
            exe_name,
            // options: Options::parse(args),
        })
    }
}

#[derive(Default)]
enum Command {
    Run(PathBuf),
    #[default]
    Help,
}

impl Command {
    fn parse(args: &mut impl Iterator<Item = String>) -> Result<Command, Fail> {
        match args.next() {
            Some(command) => match command.as_str() {
                "run" => match args.next() {
                    Some(path) => Ok(Self::Run(path.into())),
                    None => Err(fail::no_path(command)),
                },
                "help" => Ok(Self::Help),
                _ => Err(fail::bad_command(command)),
            },
            None => Ok(Self::Help),
        }
    }
}
