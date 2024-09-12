use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct ClangCXX;

impl ClangCXX {
    // by default, we will avoid writing the cxx file unless an emit flag is used
    pub fn compile(source: &str, output_file: &str) -> std::io::Result<()> {
        let mut clang = Command::new("clang++")
            .arg("-x")
            .arg("c++")
            .arg("-") // read from stdin
            .arg("-o")
            .arg(output_file)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit()) // forward stdout
            .stderr(Stdio::inherit()) // forward stderr
            .spawn()?;

        // pipe the source to the clang++ process stdin
        if let Some(ref mut stdin) = clang.stdin {
            stdin.write_all(source.as_bytes())?;
        }

        clang.wait()?;

        Ok(())
    }
}
