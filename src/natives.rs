use std::ffi::CStr;
use std::num::NonZeroU64;

use memhack_derive::foreign_fn;

// 48 83 EC 38 48 8B 11 48 8D 4C 24 20 E8
pub const GET_NAME_RVA: usize = 0x1A5540;
// 48 83 EC 40 48 8B 02 4C 8B F2 44 0F b7 7A 60
pub const CALL_FUNC_RVA: usize = 0x27A410;
// 48 8D 68 A1 48 81 EC A0 00 00 00 0F B6 F1
pub const CRASH_FUNC_RVA: usize = 0x2B3E530;

#[foreign_fn(GET_NAME_RVA)]
fn get_cname(nam: &CName) -> *const i8 {}

#[derive(Debug)]
#[repr(C)]
pub struct StackFrame {
    pub code: *const u8,
    pub func: *const Func,
    unk1: usize,
    unk2: usize,
    unk3: usize,
    unk4: usize,
    unk5: usize,
    unk6: usize,
    context: usize,
    pub parent: *mut StackFrame,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CName(NonZeroU64);

impl CName {
    pub fn resolve(&self) -> &'static str {
        unsafe { CStr::from_ptr(get_cname(self)).to_str().unwrap() }
    }
}

#[derive(Debug)]
pub struct Func {
    pub vft: *const FuncVft,
    pub name: CName,
}

#[derive(Debug)]
pub struct Class {
    _vft: usize,
    _unk1: usize,
    _parent: *const Class,
    pub name: CName,
}

#[derive(Clone, Copy)]
pub struct FuncVft {
    _get_alloc: fn(&Func) -> usize,
    _destroy: fn(&mut Func),
    pub get_class: fn(&Func) -> Option<&Class>,
}
