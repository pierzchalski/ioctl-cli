use std::os::raw::c_uchar;

use libc::{c_char, c_int, c_uint};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Int(Int),
    Struct(Struct),
    Void(Void),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Int {
    Char(c_char),
    UChar(c_uchar),
    Int(c_int),
    UInt(c_uint),
    U8(u8),
    I8(i8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Void;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Struct {
    pub fields: Vec<(String, Value)>,
}
