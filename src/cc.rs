use crate::emit::{EmitContextImpl, EmitDefault};
use crate::parser::esper_parser;

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn file_prefix(path: &PathBuf) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|s| s.to_string())
}

pub(crate) fn compile(
    input_path: PathBuf,
    output_path: PathBuf,
    clang_flags: Vec<String>,
    use_prelude: bool,
    should_emit: bool,
) {
    let source = fs::read_to_string(&input_path).expect("input file not found");

    match esper_parser::program(&source) {
        Ok(program) => {
            // dbg!(&program);
            let mut ctx = EmitContextImpl::new();
            ctx.use_prelude = use_prelude; // force?
            let mut emitter = EmitDefault { ctx };
            // dbg!(file_prefix(&input_path));
            let cxx_source = emitter.emit_program(&program, &file_prefix(&input_path).unwrap());
            // println!("{}", &out);

            if should_emit {
                fs::write(&output_path, &cxx_source);
            } else {
                ClangCXX::compile(&cxx_source, output_path.to_str().unwrap(), clang_flags).unwrap();
            }
        }

        Err(err) => {
            println!("{}", err);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClangCXX;

impl ClangCXX {
    // by default, we will avoid writing the cxx file unless an emit flag is used
    pub fn compile(
        cxx_source: &str,
        output_file: &str,
        clang_flags: Vec<String>,
    ) -> std::io::Result<()> {
        let mut clang = Command::new("clang++")
            .arg("-x")
            .arg("c++")
            .arg("-") // read from stdin
            .arg("-o")
            .arg(output_file)
            .args(clang_flags)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit()) // forward stdout
            .stderr(Stdio::inherit()) // forward stderr
            .spawn()?;

        // pipe the source to the clang++ process stdin
        if let Some(ref mut stdin) = clang.stdin {
            stdin.write_all(cxx_source.as_bytes())?;
        }

        clang.wait()?;

        Ok(())
    }
}
