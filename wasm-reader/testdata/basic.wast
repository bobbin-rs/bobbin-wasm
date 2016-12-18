(module
  (memory 1)
  (func $f (param i32 i32) (result i32)
    i32.const 0
    i32.const 0
    i32.load
    i32.const 1
    i32.add
    i32.store
    get_local 0
    get_local 1
    i32.add)
  (export "f" (func $f)))
