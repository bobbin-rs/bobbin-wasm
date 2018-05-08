.PHONY: run clean

OUT=wasm_blinky

all: $(OUT).wasm

$(OUT).wasm: src/lib.rs
	cargo build --release --target wasm32-unknown-unknown
	wasm-gc target/wasm32-unknown-unknown/release/$(OUT).wasm -o $(OUT).gc.wasm
	wasm-opt -Os $(OUT).gc.wasm -o $(OUT).wasm

run: $(OUT).wasm
	wasm-board $(OUT).wasm

clean:
	cargo clean
	rm -f $(OUT).wasm $(OUT).gc.wasm