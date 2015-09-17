extern crate libc;

use std::ffi;
use std::mem;

type ID = libc::uintptr_t;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Id(ID);

type VALUE = libc::uintptr_t;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Value(VALUE);

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
        module: VALUE,
        name: *const libc::c_char,
        superclass: VALUE)
        -> VALUE;
    fn rb_define_module_under(
        module: VALUE,
        name: *const libc::c_char)
        -> VALUE;

    fn rb_define_class(
        name: *const libc::c_char,
        superclass: VALUE)
        -> VALUE;
    fn rb_define_module(
        name: *const libc::c_char)
        -> VALUE;

    fn rb_singleton_class(
        class: VALUE)
        -> VALUE;

    fn rb_define_method_id(
        class: VALUE,
        name: ID,
        method: *const (),
        arity: libc::c_int);

    fn rb_intern2(
        name: *const libc::c_char,
        length: libc::c_long,
    ) -> ID;

    static rb_mKernel: VALUE;
    static rb_mEnumerable: VALUE;
    static rb_cObject: VALUE;
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
    flags: Value,
    class: Value,
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

static T_MASK: VALUE = 0x1f;

static FLONUM_MASK: VALUE = 0x03;
static FLONUM_FLAG: VALUE = 0x02;
static SPECIAL_MASK: VALUE = 0xff;
static SYMBOL_FLAG: VALUE = 0x0c;

static QNIL: VALUE = 0x08;
static QTRUE: VALUE = 0x14;
static QFALSE: VALUE = 0x00;
static QUNDEF: VALUE = 0x34;

pub static NIL: Value = Value(*&QNIL);
pub static TRUE: Value = Value(*&QTRUE);
pub static FALSE: Value = Value(*&QFALSE);
pub static UNDEF: Value = Value(*&QUNDEF);

fn immediate_p(value: VALUE) -> bool {
    value & 0x7 != 0
}

fn fixnum_p(value: VALUE) -> bool {
    value & 0x1 == 0x1
}

fn flonum_p(value: VALUE) -> bool {
    value & FLONUM_MASK == FLONUM_FLAG
}

fn truthy_p(value: VALUE) -> bool {
    value & QNIL == 0
}

fn static_sym_p(value: VALUE) -> bool {
    value & SPECIAL_MASK == SYMBOL_FLAG
}

unsafe fn builtin_type(value: VALUE) -> T {
    let basic: *const RBasic = mem::transmute(value);
    let Value(flags) = (*basic).flags;
    match flags & T_MASK {
        0x00 => T::None,

        0x01 => T::Object,
        0x02 => T::Class,
        0x03 => T::Module,
        0x04 => T::Float,
        0x05 => T::String,

        0x11 => T::Nil,
        0x12 => T::True,
        0x13 => T::False,
        0x14 => T::Symbol,
        0x15 => T::Fixnum,
        0x16 => T::Undef,
        _ => panic!("unknown type tag {}", flags),
    }
}

fn ruby_type(value: VALUE) -> T {
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

fn value_from_raw<'a>(value: VALUE) -> Transient<'a> {
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

    pub fn define_singleton_method<'a, M>(&'a self, name: &str, method: M) where M: Method {
        unsafe {
            let name_id = rb_intern2(name.as_ptr() as *const libc::c_char, name.len() as libc::c_long);
            let singleton_class = rb_singleton_class(mem::transmute(self));
            rb_define_method_id(mem::transmute(singleton_class), name_id, method.as_ptr(), M::arity());
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
