#![feature(libc)]

extern crate libc;

use std::ffi;
use std::mem;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Id(libc::uintptr_t);

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Value(libc::uintptr_t);

pub enum T {
    None = 0x00,

    Object = 0x01,
    Class = 0x02,
    Module = 0x03,
    String = 0x05,

    Fixnum = 0x15,
}

extern {
    fn rb_define_class_under(
        module: Value,
        name: *const libc::c_char,
        superclass: Value)
        -> Value;
    fn rb_define_module_under(
        module: Value,
        name: *const libc::c_char)
        -> Value;

    fn rb_define_class(
        name: *const libc::c_char,
        superclass: Value)
        -> Value;
    fn rb_define_module(
        name: *const libc::c_char)
        -> Value;

    fn rb_define_method_id(
        class: Value,
        name: Id,
        method: *const (),
        arity: libc::c_int);

    fn rb_intern2(
        name: *const libc::c_char,
        length: libc::c_long,
    ) -> Id;

    static rb_mKernel: Value;
    static rb_mEnumerable: Value;
    static rb_cObject: Value;
}

pub enum Transient<'a> {
    None,
    Object(&'a RObject),
    Class(&'a RClass),
    Module(&'a RClass),

    Nil,
    True,
    False,
    Symbol(isize),
    Fixnum(isize),
}

#[repr(C)]
pub struct RBasic;

#[repr(C)]
pub struct RObject {
    basic: RBasic,
}

#[repr(C)]
pub struct RClass {
    basic: RBasic,
}

#[repr(C)]
pub struct RModule {
    basic: RBasic,
}

fn fixnum_p(value: Value) -> bool {
    let Value(raw) = value;
    raw & 0x1 != 0
}

fn ruby_type(value: Value) -> T {
    if fixnum_p(value) {
        return T::Fixnum;
    }

    return T::Class;
}

fn value_from_raw<'a>(value: Value) -> Transient<'a> {
    unsafe {
        match ruby_type(value) {
            T::None => panic!("lol"),
            T::Object => Transient::Object(mem::transmute(value)),
            T::Class => Transient::Class(mem::transmute(value)),
            T::Module => Transient::Class(mem::transmute(value)),
            T::String => panic!("string not implemented"),
            T::Fixnum => panic!("fixnum not implemented"),
        }
    }
}

pub trait Method {
    unsafe fn as_ptr(self) -> *const ();
    fn arity() -> i32;
}

impl Method for extern fn(Value) -> Value {
    unsafe fn as_ptr(self) -> *const () {
        mem::transmute(self)
    }

    fn arity() -> i32 {
        0
    }
}

impl Method for extern fn(Value, Value) -> Value {
    unsafe fn as_ptr(self) -> *const () {
        mem::transmute(self)
    }

    fn arity() -> i32 {
        1
    }
}

impl Method for extern fn(Value, Value, Value) -> Value {
    unsafe fn as_ptr(self) -> *const () {
        mem::transmute(self)
    }

    fn arity() -> i32 {
        2
    }
}

impl Method for extern fn(libc::c_int, *mut Value, Value) -> Value {
    unsafe fn as_ptr(self) -> *const () {
        mem::transmute(self)
    }

    fn arity() -> i32 {
        -1
    }
}

impl RClass {
    pub fn define_module<'a>(&'a self, name: &str) -> Transient<'a> {
        unsafe {
            let name_c = ffi::CString::new(name).unwrap();
            value_from_raw(rb_define_module_under(mem::transmute(self), name_c.as_ptr()))
        }
    }

    pub fn define_class<'a>(&'a self, name: &str, superclass: &RClass) -> Transient<'a> {
        unsafe {
            let name_c = ffi::CString::new(name).unwrap();
            value_from_raw(rb_define_class_under(mem::transmute(self), name_c.as_ptr(), mem::transmute(superclass)))
        }
    }

    pub fn define_method<'a, M>(&'a self, name: &str, method: M) where M: Method {
        unsafe {
            let name_id = rb_intern2(name.as_ptr() as *const libc::c_char, name.len() as libc::c_long);
            rb_define_method_id(mem::transmute(self), name_id, method.as_ptr(), M::arity());
        }
    }
}

pub fn define_module<'a>(name: &str) -> Transient<'a> {
    unsafe {
        let name_c = ffi::CString::new(name).unwrap();
        value_from_raw(rb_define_module(name_c.as_ptr()))
    }
}

pub fn define_class<'a>(name: &str, superclass: &RClass) -> Transient<'a> {
    unsafe {
        let name_c = ffi::CString::new(name).unwrap();
        value_from_raw(rb_define_class(name_c.as_ptr(), mem::transmute(superclass)))
    }
}

pub static NIL: Value = Value(0x8);

pub fn m_kernel() -> &'static RModule {
    unsafe {
        mem::transmute(rb_mKernel)
    }
}

pub fn m_enumerable() -> &'static RModule {
    unsafe {
        mem::transmute(rb_mEnumerable)
    }
}

pub fn c_object() -> &'static RClass {
    unsafe {
        mem::transmute(rb_cObject)
    }
}
