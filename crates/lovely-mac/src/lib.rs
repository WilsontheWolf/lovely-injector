use lovely_core::sys::{LuaState};
use std::ptr::null;

use lovely_core::Lovely;
use once_cell::sync::{Lazy, OnceCell};
use libc::dlsym;
use libc::RTLD_NEXT;
use libc::RTLD_DEFAULT;
use lovely_core::log;
use std::ffi::CString;
use libloading::{Library, Symbol};
use std::panic;


static RUNTIME: OnceCell<Lovely> = OnceCell::new();

//pub static SUBSTRATE: Lazy<Library> = Lazy::new(|| unsafe { Library::new("/usr/lib/libsubstrate.dylib").unwrap() }); 
#[link(name = "CydiaSubstrate", kind = "framework")]
extern "C" {
    pub fn MSHookFunction(_: *const std::ffi::c_void, _: *const std::ffi::c_void,_: *mut *mut std::ffi::c_void);
    pub fn MSFindSymbol(_:*mut std::ffi::c_void, _:*const char) -> *const std::ffi::c_void;
}
//pub static ms_findsymbol: Lazy<Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const char) -> *const std::ffi::c_void>> =
//    Lazy::new(|| unsafe { SUBSTRATE.get(b"MSFindSymbol").unwrap() });
//pub static ms_hookfunction: Lazy<Symbol<unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_void, *mut *mut std::ffi::c_void)>> =
//    Lazy::new(|| unsafe { SUBSTRATE.get(b"MSHookFunction").unwrap() });



#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn luaL_loadbuffer(
    state: *mut LuaState,
    buf_ptr: *const u8,
    size: isize,
    name_ptr: *const u8,
) -> u32 {
    log::info!("hi dad");
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
    log::info!("hi dad");
    let rt = RUNTIME.get_unchecked();
    rt.apply_buffer_patches(state, buf_ptr, size, name_ptr, mode_ptr)
}


pub static mut orig_ptr:usize = 69420;
unsafe extern "C" fn ORIG(a: *mut LuaState, b: *const u8, c: isize, d: *const u8, e: *const u8) -> u32{
    std::mem::transmute::<_, unsafe extern "C" fn (
        *mut LuaState,
        *const u8,
        isize,
        *const u8,
        *const u8,
        ) -> u32 >(orig_ptr)
        (a,b,c,d,e)
}

#[ctor::ctor]
unsafe fn construct() {
    panic::set_hook(Box::new(|x| unsafe {
        let message = format!("lovely-injector has crashed: \n{x}");
        log::error!("{message}");
    }));

    //let mut orig = RECALL;
    let rt = Lovely::init(&|a, b, c, d, e| ORIG(a, b, c, d, e));
    RUNTIME
        .set(rt)
        .unwrap_or_else(|_| panic!("Failed to instantiate runtime."));
    log::info!("hi mom");
    //log::info!("{:?}", dlsym(RTLD_NEXT, CString::new("MSFindSymbol").unwrap().as_ptr() as *const i8));
    unsafe {
        let symbol = MSFindSymbol(core::ptr::null_mut(), CString::new("_luaL_loadbuffer").unwrap().as_ptr() as *const char);
        //let new = std::mem::transmute(&luaL_loadbufferx);
        //let new = luaL_loadbufferx;// as *const std::ffi::c_void;
        let new: *const std::ffi::c_void = std::mem::transmute(luaL_loadbuffer as *const ());
        //let orig = &mut std::mem::transmute(RECALL as *const());
        log::info!("symbol: {:?} new: {:?}, RECALL: {:?}, orig: {:?}", symbol, new, ORIG as *const(), orig_ptr);
        MSHookFunction(symbol,
            new,
            &mut orig_ptr as *mut usize as _);
        log::info!("symbol: {:?} new: {:?}, RECALL: {:?}, orig: {:?}", symbol, new, ORIG as *const(), orig_ptr);
    };
}
