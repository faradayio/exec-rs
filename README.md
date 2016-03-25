# `exec`: A Rust library to replace the running program with another

[![Latest version](https://img.shields.io/crates/v/exec.svg)](https://crates.io/crates/exec) [![License](https://img.shields.io/crates/l/exec.svg)](http://www.apache.org/licenses/LICENSE-2.0) [![Build Status](https://travis-ci.org/faradayio/exec-rs.svg?branch=master)](https://travis-ci.org/faradayio/exec-rs)

[Documentation](http://faradayio.github.io/exec-rs/exec/index.html)

This is a simple Rust wrapper around `execvp`.  It can be used as follows:

```rust
let err = exec::execvp("echo", &["echo", "foo"]);
println!("Error: {}", err);
```

We pass `"echo"` twice: Once as the name of the program we want the
operating system to execute, and once as the executed program's `argv[0]`.
Note that if `execvp` returns, it will always return an error.

If we want to treat our command line arguments as program to be run, we
could do it as follows:

```rust
// Get our command line args, dropping the first one.
let args: Vec<String> = env::args().skip(1).collect();
let program = args[0].clone();

let err = exec::execvp(program, &args);
println!("Error: {}", err);
```
