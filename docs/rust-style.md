# Rust conventions

Use the user-supplied [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html).
This is a Cargo application. Chromium's GN/crate-layout, thread and async restrictions,
Fuchsia's mutex requirement, and their test-framework rules do not apply here.

- Keep authored source files within 250 lines. `vendor/` retains upstream layout.
- Keep module indexes free of implementation; name leaf modules for their type or responsibility.
- Import from one module per `use`. Import types by name, qualify external free
  functions by module, and scope single-use trait imports to their function.
- Avoid glob imports except `super::*` in inline test modules and the guide's enum exception.
- Prefer borrows, named fields, explicit focus/action enums and inherent methods.
  Do not introduce traits merely to distribute implementation across files.
- Prefer pattern matching to check/unwrap. Use `expect` only for explained invariants;
  return recoverable I/O and validation failures with their cause.
- Fallible application constructors use `try_new`. Keep public APIs documented where
  documentation explains a contract, invariant, failure or recovery behavior.
- Use stable Rust and rustfmt with 100-column formatting. Never skip formatting to
  squeeze a file below the limit. Do not add general-purpose macros or abstractions.
- `Arc` is appropriate where eGUI's font and screenshot APIs require shared ownership.
  The diagnostic harness also shares timing data across asynchronous callbacks;
  application state itself uses unique ownership.

Run `./scripts/check.sh`. The file-size guard checks physical lines, including blank
lines and comments. Style checks complement direct review; they do not prove every
semantic recommendation in the guide automatically.
