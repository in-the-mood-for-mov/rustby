#![feature(libc)]

extern crate libc;

use std::ffi;
use std::mem;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Id(libc::uintptr_t);

type RawValue = libc::uintptr_t;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Value(RawValue);

pub enum T {
    None = 0x00,

    Object = 0x01,
    Class = 0x02,
    Module = 0x03,
    Float = 0x04,
    String = 0x05,

    Nil = 0x11,
    True = 0x12,
    False = 0x13,
    Symbol = 0x14,
    Fixnum = 0x15,
    Undef = 0x16,
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

#[derive(Debug)]
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
#[derive(Debug)]
pub struct RBasic {
    flags: RawValue,
}

#[repr(C)]
#[derive(Debug)]
pub struct RObject {
    basic: RBasic,
}

#[repr(C)]
#[derive(Debug)]
pub struct RClass {
    basic: RBasic,
}

static T_MASK: RawValue = 0x1f;

static FLONUM_MASK: RawValue = 0x03;
static FLONUM_FLAG: RawValue = 0x02;
static SPECIAL_MASK: RawValue = 0xff;
static SYMBOL_FLAG: RawValue = 0x0c;

pub static QNIL: Value = Value(0x08);
pub static QTRUE: Value = Value(0x14);
pub static QFALSE: Value = Value(0x00);
pub static QUNDEF: Value = Value(0x34);

fn immediate_p(value: Value) -> bool {
    let Value(raw) = value;
    raw & 0x7 != 0
}

fn fixnum_p(value: Value) -> bool {
    let Value(raw) = value;
    raw & 0x1 == 0x1
}

fn flonum_p(value: Value) -> bool {
    let Value(raw) = value;
    raw & FLONUM_MASK == FLONUM_FLAG
}

fn truthy_p(value: Value) -> bool {
    let Value(raw) = value;
    let Value(raw_nil) = QNIL;
    raw & !raw_nil == 0
}

fn static_sym_p(value: Value) -> bool {
    let Value(raw) = value;
    raw & SPECIAL_MASK == SYMBOL_FLAG
}

unsafe fn builtin_type(value: Value) -> T {
    let basic: *const RBasic = mem::transmute(value);
    mem::transmute(((*basic).flags & T_MASK) as u8)
}

fn ruby_type(value: Value) -> T {
    if immediate_p(value) {
        if fixnum_p(value) {
            return T::Fixnum;
        } else if flonum_p(value) {
            return T::Float;
        } else if value == QTRUE {
            return T::True;
        } else if static_sym_p(value) {
            return T::Symbol;
        } else if value == QUNDEF {
            return T::Undef;
        }
    } else if !truthy_p(value) {
        if value == QNIL {
            return T::Nil;
        } else if value == QFALSE {
            return T::False;
        }
    }

    unsafe { builtin_type(value) }
}

fn value_from_raw<'a>(value: Value) -> Transient<'a> {
    unsafe {
        match ruby_type(value) {
            T::None => panic!("lol"),
            T::Object => Transient::Object(mem::transmute(value)),
            T::Class => Transient::Class(mem::transmute(value)),
            T::Module => Transient::Module(mem::transmute(value)),
            T::Float => panic!("float not implemented"),
            T::String => panic!("string not implemented"),

            T::Nil => Transient::Nil,
            T::True => Transient::True,
            T::False => Transient::False,
            T::Symbol => panic!("symbol not implemented"),
            T::Fixnum => panic!("fixnum not implemented"),
            T::Undef => panic!("undef not implemented"),
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

pub fn define_module<'a>(name: &str) -> &'static RClass {
    unsafe {
        let name_c = ffi::CString::new(name).unwrap();
        let result = rb_define_module(name_c.as_ptr());
        match value_from_raw(result) {
            Transient::Module(class) => class,
            value@_ => panic!("expected Module, got {:?}", value),
        }
    }
}

pub fn define_class<'a>(name: &str, superclass: &RClass) -> &'static RClass {
    unsafe {
        let name_c = ffi::CString::new(name).unwrap();
        let result = rb_define_class(name_c.as_ptr(), mem::transmute(superclass));
        match value_from_raw(result) {
            Transient::Class(class) => class,
            value@_ => panic!("expected Class, got {:?}", value),
        }
    }
}

pub fn m_kernel() -> &'static RClass {
    unsafe { mem::transmute(rb_mKernel) }
}

pub fn m_enumerable() -> &'static RClass {
    unsafe { mem::transmute(rb_mEnumerable) }
}

pub fn c_object() -> &'static RClass {
    unsafe { mem::transmute(rb_cObject) }
}
