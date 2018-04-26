# Testing

bobbin-wasm leverages the [wabt](https://github.com/WebAssembly/wabt) test suite by implementing binaries namded
`wasm-objdump` and `wasm-interp` that generate output byte-for-byte identical to their wabt counterparts.

## Preparing the test environment

Download and install the current version of [wabt](https://github.com/WebAssembly/wabt), and make a symlink from
./test/ to wabt/test.

Create a local directory `./bin/`.

```
# mkdir ./bin/
```

Add symlinks for all of the wabt binaries to the ./bin directory:

```
ln -s path_to_wabt/bin/ bin/
```

Build and install the `wasm-objdump` and `wasm-interp` binaries into your test-bin directory.

```
$ cargo install --path . --root .
```

## Running Tests

To run a subset of the test suite, use `make test`. You can also run `make test-dump` or `make test-interp` to
run just the `wasm-objdump` or `wasm-interp` tests.

You should see something like this as the result:

```
$ make test
cargo -q install --path . --root . --force
cat test-dump.txt | grep -v \# | xargs test/run-tests.py --bindir /Users/jcsoo/.cargo/bin
[+71|-0|%100] (0.96s) test/dump/unreachable.txt
cat test-interp.txt | grep -v \# | xargs test/run-tests.py --bindir /Users/jcsoo/.cargo/bin
[+19|-0|%100] (0.18s) test/interp/unreachable.txt
$
```

## Configuring Tests

bobbin-wasm doesn't currently support all the functionality of the real versions of `wasm-objdump` and `wasm-interp`, so
testing them against the full test suite would produce an unreasonable number of failures.

Instead, the files [test-dump.txt](./test-dump.txt) and [test-interp.txt](./test-interp.txt) list the names of the tests that 
should be run for each tool. Tests that are known to fail are commented out with a hash mark in the first column.

Additionally, `local_test` contains copies of tests that have been useful during development. Some of these are tests
from the wabt test suite that have had specific unsupported functionality removed. There is not currently a way to
run these tests in an automated fashion.