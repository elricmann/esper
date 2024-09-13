#![allow(warnings, dead_code)]
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

mod cc;
mod emit;
mod parser;
mod visit;

use crate::cc::*;
use crate::parser::esper_parser;

#[derive(StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    #[structopt(short, long)]
    emit: bool,

    #[structopt(short = "cc", long = "clang-flags", use_delimiter = true)]
    clang_flags: Vec<String>,
}

fn main() {
    let args = Opt::from_args();
    compile(args.input, args.output, args.clang_flags, args.emit);
}
