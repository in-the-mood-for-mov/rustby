#![feature(libc)]

extern crate libc;
extern crate rustby as rb;

extern fn print_hello(_: libc::size_t) -> libc::size_t {
    println!("Hello from Rust!");
    0
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn Init_hello() {
    match rb::define_module("Hello") {
        rb::Transient::Class(m_hello) => m_hello.define_method("hello", print_hello as extern fn(libc::size_t) -> libc::size_t),
        _ => (),
    }
}
