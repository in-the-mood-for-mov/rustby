#![feature(libc)]

extern crate libc;
extern crate rustby as rb;

extern fn print_hello(_: rb::Value) -> rb::Value {
    println!("Hello from Rust!");
    rb::NIL
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn Init_hello() {
    match rb::define_module("Hello") {
        rb::Transient::Class(m_hello) => m_hello.define_method("hello", print_hello as extern fn(rb::Value) -> rb::Value),
        _ => (),
    }
}
