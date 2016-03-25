# `exec`: A Rust library to replace the running program with another

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
