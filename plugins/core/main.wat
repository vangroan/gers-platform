(module
    (import "env" "print" (func $log (param $str_ptr i32) (param $str_len i32)))

    ;; declare a page of memory and expose to host
    (memory $mem 1)
    (export "memory" (memory 0))

    ;; data section writes to the global memory at the given offset
    ;; at module intantiation time.
    (data (i32.const 0) "Hello, world!")

    (func $update (export "__gers_update")
        (i32.const 0) ;; str_ptr - location in memory
        (i32.const 13) ;; str_len
        (call $log)
    )

    ;; (export "memory" (memory 1))
)