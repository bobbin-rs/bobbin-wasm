.PHONY: run clean

all: wasm_hello.wasm

wasm_hello.wasm: src/lib.rs
	cargo build --release --target wasm32-unknown-unknown
	wasm-gc target/wasm32-unknown-unknown/release/wasm_hello.wasm -o wasm_hello.gc.wasm
	wasm-opt -Os wasm_hello.gc.wasm -o wasm_hello.wasm

run: wasm_hello.wasm
	wasm-interp --run-all-exports wasm_hello.wasm

clean:
	cargo clean
	rm -f wasm_hello.wasm wasm_hello.gc.wasm