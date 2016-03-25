# `exec`: A Rust library to replace the running program with another

[![Latest version](https://img.shields.io/crates/v/exec.svg)](https://crates.io/crates/exec) [![License](https://img.shields.io/crates/l/exec.svg)](http://www.apache.org/licenses/LICENSE-2.0) [![Build Status](https://travis-ci.org/faradayio/exec-rs.svg?branch=master)](https://travis-ci.org/faradayio/exec-rs)

[Documentation](http://faradayio.github.io/exec-rs/exec/index.html)

This is a simple Rust wrapper around `execvp`.  It can be used as follows:

```rust
// Get our command line args, dropping the first one.
let args: Vec<String> = env::args().skip(1).collect();
let program = args[0].clone();

// Exec the specified program.  Note that we pass `args[0]` twice: Once as
// the name of the program to execute, and once as the first argument.
//
// If all goes well, this will never return.  If it does return, it  will
// always retun an error.
let err = exec::execvp(program, &args);
println!("Error: {}", err);
```
