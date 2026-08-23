# bare-jit-rs

[![Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)](https://www.rust-lang.org/)

A tiny educational x86-64 arithmetic JIT written in Rust.

This intentionally targets Linux x86-64 and the System V ABI. Unsupported platforms are rejected at compile time. It emits handwritten machine-code bytes, places them in RW memory, changes the page to RX, and calls it as an `extern "C" fn(i64) -> i64`.

The library exposes compiled programs through an opaque `CompiledExpression` type. This prevents callers from passing arbitrary bytes to the execution API.

```sh
cargo run -- '(x + 3) * 7 - 2' 10
# 89
cargo test
```

Supported syntax: decimal integers, `x`, parentheses, unary `+`/`-`, and binary `+`, `-`, `*`, `/`.

This is a learning project, not production JIT infrastructure. Division by zero and the signed division overflow case intentionally retain native hardware behavior.
