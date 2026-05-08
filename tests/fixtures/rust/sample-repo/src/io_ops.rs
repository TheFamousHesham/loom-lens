// IO-effect patterns.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn write_log(path: &Path, msg: &str) -> std::io::Result<()> {
    // expect: IO=definite
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{msg}")
}

pub fn read_config(path: &Path) -> std::io::Result<String> {
    // expect: IO=definite
    fs::read_to_string(path)
}

pub fn emit(line: &str) {
    // expect: IO=definite
    println!("{line}");
}

pub fn emit_err(line: &str) {
    // expect: IO=definite
    eprintln!("{line}");
}

pub fn remove_temp(path: &Path) -> std::io::Result<()> {
    // expect: IO=definite
    fs::remove_file(path)
}

pub fn count_lines(path: &Path) -> std::io::Result<usize> {
    // expect: IO=definite
    let f = File::open(path)?;
    let r = BufReader::new(f);
    Ok(r.lines().count())
}

pub fn dump_path(path: &Path) -> String {
    // expect: IO=probable
    // Takes &Path but does not actually read; rule is a Path-parameter heuristic at probable.
    path.display().to_string()
}
