use std::collections::VecDeque;
use std::ffi::{c_char, c_int, c_void, CString};
use std::ptr;
use std::slice;

use itertools::Itertools;
pub use mlua_sys::*;
use log::info;


pub type LuaState = lua_State; // TODO: Remove this shim
pub type LuaFunc = lua_CFunction;

pub const LUA_GLOBALSINDEX: c_int = -10002;
pub const LUA_TNIL: c_int = 0;
pub const LUA_TBOOLEAN: c_int = 1;
pub const fn lua_upvalueindex(i: c_int) -> c_int {
    // This is a macro in lua
    LUA_GLOBALSINDEX - i
}

// TODO: Can we make this work with variable number of upvalues?
unsafe extern "C-unwind" fn lua_return_values(state: *mut LuaState) -> c_int {
    let index = lua_upvalueindex(1);
    lua_pushvalue(state, index);
    1
}

// HACK: Panics if not inlined?
#[inline(always)]
pub(crate) unsafe fn check_lua_string(state: *mut LuaState, index: c_int) -> String {
    let mut str_len = 0usize;
    let arg_str = luaL_checklstring(state, index, &mut str_len);

    let str_buf = slice::from_raw_parts(arg_str as *const u8, str_len);
    String::from_utf8_lossy(str_buf).to_string()
}

// TODO: implement all lua methods on this(?)
pub(crate) trait LuaStateTrait {
    unsafe fn push<P: Pushable>(self, obj: P);
    unsafe fn push_closure(self, func: LuaFunc, vals: c_int);
    unsafe fn to_string(self, index: c_int) -> String;
}

impl LuaStateTrait for *mut LuaState {
    unsafe fn push<P: Pushable>(self, obj: P) {
        obj.push(self)
    }

    unsafe fn push_closure(self, func: LuaFunc, vals: c_int) {
        lua_pushcclosure(self, func, vals);
    }

    unsafe fn to_string(self, index: c_int) -> String {
        let mut str_len = 0usize;
        let arg_str = lua_tolstring(self, index, &mut str_len);

        let str_buf = slice::from_raw_parts(arg_str as *const u8, str_len);
        String::from_utf8_lossy(str_buf).to_string()
    }
}
/// A trait which allows the implementing value to generically push its value onto the Lua stack.
pub trait Pushable {
    /// Push this value onto the Lua stack.
    ///
    /// # Safety
    /// Directly interacts with native Lua state.
    unsafe fn push(&self, state: *mut LuaState);
}

impl Pushable for String {
    unsafe fn push(&self, state: *mut LuaState) {
        let value = format!("{self}\0");
        lua_pushstring(state, value.as_ptr() as _);
    }
}

impl Pushable for &String {
    unsafe fn push(&self, state: *mut LuaState) {
        let value = format!("{self}\0");
        lua_pushstring(state, value.as_ptr() as _);
    }
}

impl Pushable for &str {
    unsafe fn push(&self, state: *mut LuaState) {
        let value = CString::new(*self).unwrap();
        lua_pushstring(state, value.into_raw() as _);
    }
}

impl Pushable for isize {
    unsafe fn push(&self, state: *mut LuaState) {
        lua_pushnumber(state, *self as _);
    }
}

impl Pushable for bool {
    unsafe fn push(&self, state: *mut LuaState) {
        lua_pushboolean(state, *self as _);
    }
}

impl Pushable for LuaFunc {
    unsafe fn push(&self, state: *mut LuaState) {
        lua_pushcclosure(state, *self as _, 0);
    }
}

pub struct LuaVar<P>
where
    P: std::ops::Deref,
    P::Target: Pushable,
{
    name: String,
    val: P,
}

pub struct LuaTable {
    var: Vec<LuaVar<Box<dyn Pushable>>>,
}

impl LuaTable {
    pub(crate) fn new() -> Self {
        LuaTable { var: vec![] }
    }

    /// Add a variable to this Lua module.
    pub fn add_var<P: Pushable + 'static>(self, name: &'static str, val: P) -> Self {
        let name = format!("{name}\0");
        let mut var = self.var;
        let val = Box::new(val);
        var.push(LuaVar { name, val });

        LuaTable { var }
    }
}

impl Pushable for LuaTable {
    unsafe fn push(&self, state: *mut LuaState) {
        // Create a table at the top of the stack.
        lua_createtable(state, 0, self.var.len().try_into().unwrap());

        for lua_var in self.var.iter() {
            // Push the var name and value onto the stack.
            lua_pushstring(state, lua_var.name.as_ptr() as _);
            lua_var.val.push(state);

            // Set the table key:val from what we previously pushed onto the stack.
            lua_settable(state, -3);
        }
    }
}

/// Commit this Lua module to native Lua state.
///
/// # Safety
/// Directly interacts and mutates native Lua state.
pub unsafe fn preload_module<P: Pushable>(state: *mut LuaState, name: &'static str, value: P) {
    let top = lua_gettop(state);

    // Get the package.preloads
    lua_getfield(state, LUA_GLOBALSINDEX, c"package".as_ptr());
    lua_getfield(state, -1, c"preload".as_ptr());
    let preload_index = lua_gettop(state);

    // Push the value to the stack
    value.push(state);

    // Push our function to the stack
    let func: LuaFunc = lua_return_values;

    state.push_closure(func, 1);

    let name = CString::new(name).unwrap();
    lua_setfield(state, preload_index, name.into_raw() as _);

    // Reset the stack.
    lua_settop(state, top);
}

/// Load the provided buffer as a lua module with the specified name.
/// # Safety
/// Makes a lot of FFI calls, mutates internal C lua state.
pub unsafe fn load_module<F: Fn(*mut LuaState, *const u8, usize, *const u8, *const u8) -> u32>(
    state: *mut LuaState,
    name: &str,
    buffer: &str,
    lual_loadbufferx: &F,
) {
    let p_name = format!("@{name}");
    let p_name_cstr = CString::new(p_name).unwrap();

    // Push the global package.preload table onto the top of the stack, saving its index.
    let stack_top = lua_gettop(state);
    lua_getfield(state, LUA_GLOBALSINDEX, c"package".as_ptr());
    lua_getfield(state, -1, c"preload".as_ptr());

    // This is the index of the `package.loaded` table.
    let field_index = lua_gettop(state);

    // Load the buffer and execute it via lua_pcall, pushing the result to the top of the stack.
    lual_loadbufferx(
        state,
        buffer.as_ptr(),
        buffer.len(),
        p_name_cstr.into_raw() as _,
        ptr::null(),
    );

    let lua_pcall_return = lua_pcall(state, 0, -1, 0);
    if lua_pcall_return == 0 {
        lua_pushcclosure(state, lua_identity_closure, 1);
        // Insert wrapped pcall results onto the package.preload global table.
        let module_cstr = CString::new(name).unwrap();

        lua_setfield(state, field_index, module_cstr.into_raw());
    }

    lua_settop(state, stack_top);
}

// Checks if a module is in the preload table. Used to check if lovely was already initalized
// # Safety
// Uses the native lua API. I'm also pretty sure I it bikes without a helmet.
pub(crate) unsafe fn is_module_preloaded(state: *mut LuaState, name: &str) -> bool {
    let name_cstr = CString::new(name).unwrap();
    let stack_top = lua_gettop(state);
    lua_getfield(state, LUA_GLOBALSINDEX, c"package".as_ptr());
    lua_getfield(state, -1, c"preload".as_ptr());
    lua_getfield(state, -1, name_cstr.as_ptr());

    let res = lua_type(state, -1) != LUA_TNIL;

    lua_settop(state, stack_top);
    res
}

/// An override print function, copied piecemeal from the Lua 5.1 source, but in Rust.
/// # Safety
/// Native lua API access. It's unsafe, it's unchecked, it will probably eat your firstborn.
pub unsafe extern "C-unwind" fn override_print(state: *mut LuaState) -> c_int {
    let argc = lua_gettop(state);
    let mut out = VecDeque::new();

    for _ in 0..argc {
        // We call Lua's builtin tostring function because we don't have access to the 5.3 luaL_tolstring
        // helper function. It's not pretty, but it works.
        lua_getfield(state, LUA_GLOBALSINDEX, c"tostring".as_ptr());
        lua_pushvalue(state, -2);
        lua_call(state, 1, 1);

        let mut str_len = 0usize;
        let arg_str = lua_tolstring(state, -1, &mut str_len);

        let str_buf = slice::from_raw_parts(arg_str as *const u8, str_len);
        let arg_str = String::from_utf8_lossy(str_buf).to_string();

        out.push_front(arg_str);
        lua_settop(state, -3);
    }

    let msg = out.into_iter().join("\t");

    info!("[G] {msg}");

    0
}

/// A function, which as a Lua closure, returns the first upvalue. This lets it
/// be used to wrap lua values into a closure which returns that value.
/// # Safety
/// Makes some FFI calls, mutates internal C lua state.
pub unsafe extern "C-unwind" fn lua_identity_closure(state: *mut LuaState) -> c_int {
    // LUA_GLOBALSINDEX - 1 is where the first upvalue is located
    lua_pushvalue(state, LUA_GLOBALSINDEX - 1);
    // We just return that value
    1
}

/// A function, which as a Lua closure, throws the first upvalue. This lets it
/// be used to wrap lua values into a closure which throws that value.
/// # Safety
/// Makes some FFI calls, mutates internal C lua state.
pub unsafe extern "C-unwind" fn lua_err_identity_closure(state: *mut LuaState) -> c_int {
    // LUA_GLOBALSINDEX - 1 is where the first upvalue is located
    lua_pushvalue(state, LUA_GLOBALSINDEX - 1);
    lua_error(state)
}
