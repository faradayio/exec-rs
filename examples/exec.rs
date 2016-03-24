extern crate exec;

use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let program = args[0].clone();
    exec::execvp(program, &args).unwrap();
}
