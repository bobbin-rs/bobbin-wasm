# bobbin-wasm

**bobbin-wasm** is a [WebAssembly](http://webassembly.org) library and interpreter written
using the [Rust](https://www.rust-lang.org) programming language, designed to run in
resource constrained embedded systems such as [ARM Cortex-M](https://www.arm.com/products/processors/cortex-m)
SOCs with limited flash, RAM, and processing power, but also also to be useful as a general
purpose interpreter to be embedded in applications for scripting and sandboxing purposes.

To achieve those goals, **bobbin-wasm** does not depend on the Rust standard library and does
not require an allocator. It is #[no_std] by default, though future versions could add opt-in
features that make use of the standard library. It is also planned to make the entire parser,
validator, and compiler / interpreter panic-free so that it is straightforward to use the
system as a C library after wrappers are written.

To read more about these goals, see [MOTIVATION](./MOTIVATION.md).

The current version is based heavily on [WABT](https://github.com/WebAssembly/wabt), particularly
the validation and typechecking components which are straight ports of their C++ counterparts.

In fact, for testing and validation purposes, **bobbin-wasm** implements clones of `wasm-objdump` and
`wasm-interp` that produce output that is byte-for-byte identical to the originals, allowing the
use of the extensive `WABT` test suite. See [TESTING](./TESTING.md) for more details.

## Building

To build the library and binaries, use `cargo build`.

To run the object dumper directly without installing, use `cargo run --bin wasm-objdump`. To run
the interpreter directly without installing, use `cargo run --bin wasm-interp`.

To install the binaries, use `cargo install` or `cargo install --force` to overwrite your existing
binaries.

## Hello Wasm

```
$ cargo install --force
$ cd wasm-hello
$ make run
   Compiling wasm-hello v0.1.0 (file:///Users/jcsoo/bobbin-dev/bobbin-wasm/wasm-hello)
    Finished release [optimized] target(s) in 1.11s
wasm-gc target/wasm32-unknown-unknown/release/wasm_hello.wasm -o wasm_hello.gc.wasm
wasm-opt -Os wasm_hello.gc.wasm -o wasm_hello.wasm
wasm-interp --run-all-exports wasm_hello.wasm
Hello, World
run_hello() =>
```

## Current Status

**bobbin-wasm** should be considered *extremely unstable*.

### Memory and Resource Limits

The biggest limitation is that there are many memory and resource limits that are hard coded and set 
high enough to run the test suite, but not nearly high enough to run typical WASM binaries produced by the
current Rust and C toolchains, even after optimization. These limits will be gradually changed to be
configurable.

### Instruction Subset

Currently, only 32-bit integer instructions are fully implemented. The parser and validator should
recognize 32-bit floating point instructions but they will not execute in the interpreter.

Eventually the goal is to provide support for 32-bit integer and floating point with a compile-time
option for 32-bit integer only.

### Host API

The host API is extremely crude and should be considered proof of concept. Eventually there should be
a low-level API as well as higher-level APIs and macros and codegen tools to support type-safe
Rust API implementation.


### Documentation and Examples

Documentation is currently very sparse. [wasm-interp](src/bin/wasm-interp.rs) is the best starting point for anyone
that wants build a simple application and or to use the host API.

Cross-platform examples for running an interpreter on embedded devices will be released as soon as the underlying
hardware crates are released.



