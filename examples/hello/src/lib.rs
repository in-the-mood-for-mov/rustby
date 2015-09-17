extern crate libc;
extern crate rustby as rb;

extern fn print_hello(_: rb::Value) -> rb::Value {
    println!("Hello from Rust!");
    rb::NIL
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern fn Init_hello() {
    rb::define_module("Hello")
        .define_method("hello", print_hello as extern fn(rb::Value) -> rb::Value)
}
