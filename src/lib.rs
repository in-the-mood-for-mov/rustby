#![feature(libc)]

extern crate libc;

use std::ffi;
use std::mem;

pub enum T {
    None = 0x00,

    Object = 0x01,
    Class = 0x02,
    Module = 0x03,
    String = 0x05,

    Fixnum = 0x15,
}

extern {
    pub fn rb_define_class_under(
        module: libc::size_t,
        name: *const libc::c_char,
        superclass: libc::size_t)
        -> libc::size_t;
    pub fn rb_define_module_under(
        module: libc::size_t,
        name: *const libc::c_char)
        -> libc::size_t;

    pub fn rb_define_class(
        name: *const libc::c_char,
        superclass: libc::size_t)
        -> libc::size_t;
    pub fn rb_define_module(
        name: *const libc::c_char)
        -> libc::size_t;

    pub static rb_mKernel: libc::size_t;
    pub static rb_mEnumerable: libc::size_t;
    pub static rb_cObject: libc::size_t;
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

fn fixnum_p(value: libc::size_t) -> bool {
    value & 0x1 != 0
}

pub fn ruby_type(value: libc::size_t) -> T {
    if fixnum_p(value) {
        return T::Fixnum;
    }

    return T::Class;
}

fn value_from_raw<'a>(value: libc::size_t) -> Transient<'a> {
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
