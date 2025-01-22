use std::{
    collections::VecDeque,
    ffi::{c_void, CString},
    ptr, slice,
};

use itertools::Itertools;
use libloading::{Library, Symbol};
use log::info;
use once_cell::sync::Lazy;

pub const LUA_GLOBALSINDEX: isize = -10002;

pub const LUA_TNIL: isize = 0;
pub const LUA_TBOOLEAN: isize = 1;

pub type LuaState = c_void;

extern "C" {
    fn get_call() -> *const std::ffi::c_void;
    fn get_pcall() -> *const std::ffi::c_void;
    fn get_getfield() -> *const std::ffi::c_void;
    fn get_setfield() -> *const std::ffi::c_void;
    fn get_gettop() -> *const std::ffi::c_void;
    fn get_settop() -> *const std::ffi::c_void;
    fn get_pushvalue() -> *const std::ffi::c_void;
    fn get_pushcclosure() -> *const std::ffi::c_void;
    fn get_tolstring() -> *const std::ffi::c_void;
    fn get_toboolean() -> *const std::ffi::c_void;
    fn get_topointer() -> *const std::ffi::c_void;
    fn get_type() -> *const std::ffi::c_void;
    fn get_typename() -> *const std::ffi::c_void;
    fn get_isstring() -> *const std::ffi::c_void;
}

pub static lua_call: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize, isize)>> =
    Lazy::new(|| unsafe { std::mem::transmute(get_call()) });

pub static lua_pcall: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize, isize, isize) -> isize>,> =
    Lazy::new(|| unsafe { std::mem::transmute(get_pcall()) });

pub static lua_getfield: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize, *const char)>> =
    Lazy::new(|| unsafe { std::mem::transmute(get_getfield()) });

pub static lua_setfield: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize, *const char)>> =
    Lazy::new(|| unsafe { std::mem::transmute(get_setfield()) });

pub static lua_gettop: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState) -> isize>> =
    Lazy::new(|| unsafe { std::mem::transmute(get_gettop()) });

pub static lua_settop: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> isize>> =
    Lazy::new(|| unsafe { std::mem::transmute(get_settop()) });

pub static lua_pushvalue: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize)>> =
    Lazy::new(|| unsafe { std::mem::transmute(get_pushvalue()) });

pub static lua_pushcclosure: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, *const c_void, isize)>,> =
    Lazy::new(|| unsafe { std::mem::transmute(get_pushcclosure()) });

pub static lua_tolstring: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize, *mut isize) -> *const char>,> =
    Lazy::new(|| unsafe { std::mem::transmute(get_tolstring()) });

pub static lua_toboolean: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> bool>> =
    Lazy::new(|| unsafe { std::mem::transmute(get_toboolean()) });

pub static lua_topointer: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> *const c_void>,> =
    Lazy::new(|| unsafe { std::mem::transmute(get_topointer()) });

pub static lua_type: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> isize>> =
    Lazy::new(|| unsafe { std::mem::transmute(get_type()) });

pub static lua_typename: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> *const char>> =
    Lazy::new(|| unsafe { std::mem::transmute(get_typename()) });

pub static lua_isstring: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> isize>> =
    Lazy::new(|| unsafe { std::mem::transmute(get_isstring()) });


/*#[link(name = "CydiaSubstrate", kind = "framework")]
extern "C" {
    pub fn MSFindSymbol(_:*mut std::ffi::c_void, _:*const char) -> *const std::ffi::c_void;
}

pub static lua_call: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize, isize)>> =
    Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_call").unwrap().as_ptr() as *const char)) });

pub static lua_pcall: Lazy<
    Symbol<unsafe extern "C" fn(*mut LuaState, isize, isize, isize) -> isize>,
> = Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_pcall").unwrap().as_ptr() as *const char)) });

pub static lua_getfield: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize, *const char)>> =
    Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_getfield").unwrap().as_ptr() as *const char)) });

pub static lua_setfield: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize, *const char)>> =
    Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_setfield").unwrap().as_ptr() as *const char)) });

pub static lua_gettop: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState) -> isize>> =
    Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_gettop").unwrap().as_ptr() as *const char)) });

pub static lua_settop: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> isize>> =
    Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_settop").unwrap().as_ptr() as *const char)) });

pub static lua_pushvalue: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize)>> =
    Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_pushvalue").unwrap().as_ptr() as *const char)) });

pub static lua_pushcclosure: Lazy<
    Symbol<unsafe extern "C" fn(*mut LuaState, *const c_void, isize)>,
> = Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_pushcclosure").unwrap().as_ptr() as *const char)) });

pub static lua_tolstring: Lazy<
    Symbol<unsafe extern "C" fn(*mut LuaState, isize, *mut isize) -> *const char>,
> = Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_tolstring").unwrap().as_ptr() as *const char)) });

pub static lua_toboolean: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> bool>> =
    Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_toboolean").unwrap().as_ptr() as *const char)) });

pub static lua_topointer: Lazy<
    Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> *const c_void>,
> = Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_topointer").unwrap().as_ptr() as *const char)) });

pub static lua_type: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> isize>> =
    Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_type").unwrap().as_ptr() as *const char)) });

pub static lua_typename: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> *const char>> =
    Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_typename").unwrap().as_ptr() as *const char)) });

pub static lua_isstring: Lazy<Symbol<unsafe extern "C" fn(*mut LuaState, isize) -> isize>> =
    Lazy::new(|| unsafe { std::mem::transmute(MSFindSymbol(core::ptr::null_mut(), CString::new("_lua_isstring").unwrap().as_ptr() as *const char)) });
*/

/// Load the provided buffer as a lua module with the specified name.
/// # Safety
/// Makes a lot of FFI calls, mutates internal C lua state.
pub unsafe fn load_module<F: Fn(*mut LuaState, *const u8, isize, *const u8, *const u8) -> u32>(
    state: *mut LuaState,
    name: &str,
    buffer: &str,
    lual_loadbufferx: &F,
) {
    let buf_cstr = CString::new(buffer).unwrap();
    let buf_len = buf_cstr.as_bytes().len();

    let p_name = format!("@{name}");
    let p_name_cstr = CString::new(p_name).unwrap();

    // Push the global package.preload table onto the top of the stack, saving its index.
    let stack_top = lua_gettop(state);
    lua_getfield(state, LUA_GLOBALSINDEX, b"package\0".as_ptr() as _);
    lua_getfield(state, -1, b"preload\0".as_ptr() as _);

    // This is the index of the `package.loaded` table.
    let field_index = lua_gettop(state);

    // Load the buffer and execute it via lua_pcall, pushing the result to the top of the stack.
    lual_loadbufferx(
        state,
        buf_cstr.into_raw() as _,
        buf_len as _,
        p_name_cstr.into_raw() as _,
        ptr::null(),
    );

    let lua_pcall_return = lua_pcall(state, 0, -1, 0);
    if lua_pcall_return == 0 {
        lua_pushcclosure(state, lua_identity_closure as *const c_void, 1);
        // Insert wrapped pcall results onto the package.preload global table.
        let module_cstr = CString::new(name).unwrap();

        lua_setfield(state, field_index, module_cstr.into_raw() as _);
    }

    lua_settop(state, stack_top);
}

/// An override print function, copied piecemeal from the Lua 5.1 source, but in Rust.
/// # Safety
/// Native lua API access. It's unsafe, it's unchecked, it will probably eat your firstborn.
pub unsafe extern "C" fn override_print(state: *mut LuaState) -> isize {
    let argc = lua_gettop(state);
    let mut out = VecDeque::new();

    for _ in 0..argc {
        // We call Lua's builtin tostring function because we don't have access to the 5.3 luaL_tolstring
        // helper function. It's not pretty, but it works.
        lua_getfield(state, LUA_GLOBALSINDEX, b"tostring\0".as_ptr() as _);
        lua_pushvalue(state, -2);
        lua_call(state, 1, 1);

        let mut str_len = 0_isize;
        let arg_str = lua_tolstring(state, -1, &mut str_len);

        let str_buf = slice::from_raw_parts(arg_str as *const u8, str_len as _);
        let arg_str = String::from_utf8_lossy(str_buf).to_string();

        out.push_front(arg_str);
        lua_settop(state, -(1) - 1);
    }

    let msg = out.into_iter().join("\t");

    info!("[G] {msg}");

    0
}

/// A function, which as a Lua closure, returns the first upvalue. This lets it
/// be used to wrap lua values into a closure which returns that value.
/// # Safety
/// Makes some FFI calls, mutates internal C lua state.
pub unsafe extern "C" fn lua_identity_closure(state: *mut LuaState) -> isize {
    // LUA_GLOBALSINDEX - 1 is where the first upvalue is located
    lua_pushvalue(state, LUA_GLOBALSINDEX - 1);
    // We just return that value
    return 1;
}
