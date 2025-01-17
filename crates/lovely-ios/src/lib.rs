use lovely_core::sys::LuaState;
use std::{env, ptr::null};
use std::panic;
use lovely_core::log::{*};
use std::ffi::CString;
use lovely_core::Lovely;
use once_cell::sync::OnceCell;

static RUNTIME: OnceCell<Lovely> = OnceCell::new();

#[link(name = "CydiaSubstrate", kind = "framework")]
extern "C" {
    pub fn MSHookFunction(_: *const std::ffi::c_void, _: *const std::ffi::c_void,_: *mut *mut std::ffi::c_void);
    pub fn MSFindSymbol(_:*mut std::ffi::c_void, _:*const u8) -> *const std::ffi::c_void;
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn luaL_loadbuffer(
    state: *mut LuaState,
    buf_ptr: *const u8,
    size: isize,
    name_ptr: *const u8,
) -> u32 {
    info!("luaL_loadbuffer");
    let rt = RUNTIME.get_unchecked();
    rt.apply_buffer_patches(state, buf_ptr, size, name_ptr, null())
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn luaL_loadbufferx(
    state: *mut LuaState,
    buf_ptr: *const u8,
    size: isize,
    name_ptr: *const u8,
    mode_ptr: *const u8,
) -> u32 {
    info!("luaL_loadbufferx");
    let rt = RUNTIME.get_unchecked();
    rt.apply_buffer_patches(state, buf_ptr, size, name_ptr, mode_ptr)
}

pub static mut ORIG_PTR:usize = 69420;
unsafe extern "C" fn orig(a: *mut LuaState, b: *const u8, c: isize, d: *const u8, e: *const u8) -> u32{
    std::mem::transmute::<_, unsafe extern "C" fn (
        *mut LuaState,
        *const u8,
        isize,
        *const u8,
        *const u8,
        ) -> u32 >(ORIG_PTR)
        (a,b,c,d,e)
}

#[ctor::ctor]
unsafe fn construct() {
    panic::set_hook(Box::new(|x| {
        let message = format!("lovely-injector has crashed: \n{x}");
        error!("{message}");
    }));
    let args: Vec<_> = env::args().collect();
    let dump_all = args.contains(&"--dump-all".to_string());

    let rt = Lovely::init(&|a, b, c, d, e| orig(a, b, c, d, e), dump_all);
    RUNTIME
        .set(rt)
        .unwrap_or_else(|_| panic!("Failed to instantiate runtime."));
    info!("About to hook luaL_loadbuffer");
    unsafe {
        let symbol = MSFindSymbol(core::ptr::null_mut(), CString::new("_luaL_loadbuffer").unwrap().as_ptr() as *const u8);
        let new: *const std::ffi::c_void = std::mem::transmute(luaL_loadbuffer as *const ());
        MSHookFunction(symbol,
            new,
            &mut ORIG_PTR as *mut usize as _);
    };
    info!("About to hook luaL_loadbufferx");
    unsafe {
        let symbol = MSFindSymbol(core::ptr::null_mut(), CString::new("_luaL_loadbufferx").unwrap().as_ptr() as *const u8);
        let new: *const std::ffi::c_void = std::mem::transmute(luaL_loadbufferx as *const ());
        MSHookFunction(symbol,
            new,
            core::ptr::null_mut());
    };
    info!("All hooked up!");
}
