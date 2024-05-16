#![allow(non_upper_case_globals)]

use core::slice;
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::sync::Once;
use std::time::Instant;
use std::{env, fs};
use std::path::{Path, PathBuf};

use log::*;

use getargs::{Arg, Options};
use manifest::Patch;
use ropey::Rope;
use sha2::{Digest, Sha256};
use sys::LuaState;

use crate::manifest::PatchManifest;

pub mod sys;
pub mod manifest;
pub mod log;
pub mod patch;

type LoadBuffer = dyn Fn(*mut LuaState, *const u8, isize, *const u8) -> u32 + Send + Sync + 'static;

pub struct Lovely {
    pub mod_dir: PathBuf,
    pub is_vanilla: bool,
    loadbuffer: &'static LoadBuffer,
    patch_table: PatchTable,
    rt_init: Once,
}

impl Lovely {
    /// Initialize the Lovely patch runtime.
    pub fn init(loadbuffer: &'static LoadBuffer) -> Self {
        let start = Instant::now();

        let args = std::env::args().skip(1).collect::<Vec<_>>();
        let mut opts = Options::new(args.iter().map(String::as_str));
        let game_name = env::current_exe()
            .expect("Failed to get the path of the current executable.")
            .file_stem()
            .expect("Failed to get file_stem component of current executable path.")
            .to_string_lossy()
            .replace(".", "_");
        let mut mod_dir = dirs::config_dir()
            .unwrap()
            .join(game_name)
            .join("Mods");

        let log_dir = mod_dir.join("lovely").join("log");
        
        log::init(&log_dir).unwrap_or_else(|e| panic!("Failed to initialize logger: {e:?}"));
        
        let version = env!("CARGO_PKG_VERSION");
        info!("Lovely {version}");

        let mut is_vanilla = false;
    
        while let Some(opt) = opts.next_arg().expect("Failed to parse argument.") {
            match opt {
                Arg::Long("mod-dir") => mod_dir = opts.value().map(PathBuf::from).unwrap_or(mod_dir),
                Arg::Long("vanilla") => is_vanilla = true,
                _ => (),
            }
        }

        // Stop here if we're running in vanilla mode.
        if is_vanilla {
            info!("Running in vanilla mode");

            return Lovely {
                mod_dir,
                is_vanilla,
                loadbuffer,
                patch_table: Default::default(),
                rt_init: Once::new(),
            };
        }

        // Validate that an older Lovely install doesn't already exist within the game directory.
        let exe_path = env::current_exe().unwrap();
        let game_dir = exe_path.parent().unwrap();
        let dwmapi = game_dir.join("dwmapi.dll"); 

        if dwmapi.is_file() {
            panic!(
                "An old Lovely installation was detected within the game directory. \
                This problem MUST BE FIXED before you can start the game.\n\nTO FIX: Delete the file at {dwmapi:?}"
            );
        }

        info!("Game directory is at {game_dir:?}");
        info!("Writing logs to {log_dir:?}");
    
        if !mod_dir.is_dir() {
            info!("Creating mods directory at {mod_dir:?}");
            fs::create_dir_all(&mod_dir).unwrap();
        }
    
        info!("Using mod directory at {mod_dir:?}");
        let patch_table = PatchTable::load(&mod_dir)
            .with_loadbuffer(loadbuffer);

        let dump_dir = mod_dir.join("lovely").join("dump");
        if dump_dir.is_dir() {
            info!("Cleaning up dumps directory at {dump_dir:?}");
            fs::remove_dir_all(&dump_dir)
                .unwrap_or_else(|e| panic!("Failed to recursively delete dumps directory at {dump_dir:?}: {e:?}"));
        }
        
        info!("Initialization complete in {}ms", start.elapsed().as_millis());

        Lovely {
            mod_dir,
            is_vanilla,
            loadbuffer,
            patch_table,
            rt_init: Once::new(),
        }
    }

    /// Apply patches onto the raw buffer.
    /// 
    /// # Safety
    /// This function is unsafe because
    /// - It interacts and manipulates memory directly through native pointers
    /// - It interacts, calls, and mutates native lua state through native pointers
    pub unsafe fn apply_buffer_patches(&self, state: *mut LuaState, buf_ptr: *const u8, size: isize, name_ptr: *const u8) -> u32 {
        // Install native function overrides.
        self.rt_init.call_once(|| {
            let closure = sys::override_print as *const c_void;
            sys::lua_pushcclosure(state, closure, 0);
            sys::lua_setfield(state, sys::LUA_GLOBALSINDEX, b"print\0".as_ptr() as _);

            // Inject Lovely functions into the runtime.
            self.patch_table.inject_metadata(state);
        });

        let name = CStr::from_ptr(name_ptr as _).to_str()
            .unwrap_or_else(|e| panic!("The byte sequence at {name_ptr:x?} is not a valid UTF-8 string: {e:?}"));

        // Stop here if no valid patch exists for this target.
        if !self.patch_table.needs_patching(name) {
            return (self.loadbuffer)(state, buf_ptr, size, name_ptr);
        }

        // Convert the buffer from cstr ptr, to byte slice, to utf8 str.
        let buf = slice::from_raw_parts(buf_ptr, (size - 1) as _);
        let buf_str = CString::new(buf)
            .unwrap_or_else(|e| panic!("The byte buffer '{buf:?}' for target {name} contains a non-terminating null char: {e:?}"));
        let buf_str = buf_str.to_str()
            .unwrap_or_else(|e| panic!("The byte buffer '{buf:?}' for target {name} contains invalid UTF-8: {e:?}"));

        let patched = self.patch_table.apply_patches(name, buf_str, state);

        let patch_dump = self.mod_dir
            .join("lovely")
            .join("dump")
            .join(name.replace('@', ""));

        let dump_parent = patch_dump.parent().unwrap();
        if !dump_parent.is_dir() {
            fs::create_dir_all(dump_parent).unwrap();
        }

        // Write the patched file to the dump, moving on if an error occurs.
        if let Err(e) = fs::write(&patch_dump, &patched) {
            error!("Failed to write patched buffer to {patch_dump:?}: {e:?}");
        }

        let raw = CString::new(patched).unwrap();
        let raw_size = raw.as_bytes().len();
        let raw_ptr = raw.into_raw();

        (self.loadbuffer)(state, raw_ptr as _, raw_size as _, name_ptr)
    }
}

#[derive(Default)]
pub struct PatchTable {
    mod_dir: PathBuf,
    loadbuffer: Option<&'static LoadBuffer>,
    targets: HashSet<String>,
    patches: Vec<Patch>,
    vars: HashMap<String, String>,
    args: HashMap<String, String>,
}

impl PatchTable {
    /// Load patches from the provided mod directory. This scans for lovely patch files
    /// within each subdirectory that matches either:
    /// - MOD_DIR/lovely.toml
    /// - MOD_DIR/lovely/*.toml
    pub fn load(mod_dir: &Path) -> PatchTable {
        let mod_dirs = fs::read_dir(mod_dir)
            .unwrap_or_else(|e| panic!("Failed to read from mod directory within {mod_dir:?}:\n{e:?}"))
            .filter_map(|x| x.ok())
            .filter(|x| x.path().is_dir())
            .map(|x| x.path());

        let patch_files = mod_dirs
            .flat_map(|dir| {
                let lovely_toml = dir.join("lovely.toml");
                let lovely_dir = dir.join("lovely");
                let mut toml_files = Vec::new();

                if lovely_toml.is_file() {
                    toml_files.push(lovely_toml);
                }

                if lovely_dir.is_dir() {
                    let mut subfiles = fs::read_dir(&lovely_dir)
                        .unwrap_or_else(|_| panic!("Failed to read from lovely directory at '{lovely_dir:?}'."))
                        .filter_map(|x| x.ok())
                        .map(|x| x.path())
                        .filter(|x| x.is_file())
                        .filter(|x| x.extension().unwrap() == "toml")
                        .collect::<Vec<_>>();
                    toml_files.append(&mut subfiles);
                }

                toml_files
            })
            .collect::<Vec<_>>();

        let mut targets: HashSet<String> = HashSet::new();
        let mut patches: Vec<Patch> = Vec::new();
        let mut var_table: HashMap<String, String> = HashMap::new();

        // Load n > 0 patch files from the patch directory, collecting them for later processing.
        for patch_file in patch_files {
            let patch_dir = patch_file.parent().unwrap();
            
            // Determine the mod directory from the location of the lovely patch file.
            let mod_dir = if patch_dir.file_name().unwrap() == "lovely" {
                patch_dir.parent().unwrap()
            } else {
                patch_dir
            };

            let mut patch: PatchManifest = {
                let str = fs::read_to_string(&patch_file)
                    .unwrap_or_else(|e| panic!("Failed to read patch file at {patch_file:?}:\n{e:?}"));

                let ignored_key_callback = |key: serde_ignored::Path| {
                    // get the last component of the key, which looks something like patches.0.overwrite
                    if let serde_ignored::Path::Map { parent: _, key: ref key_last_component } = key
                    {
                        if key_last_component == "overwrite" {
                            warn!("The key `overwrite` is deprecated. To perform replacement use `position = \"at\"`.");
                        }
                    }
                    warn!("Unknown key `{key}` found in patch file at {patch_file:?}, ignoring it");
                };

                serde_ignored::deserialize(toml::Deserializer::new(&str), ignored_key_callback)
                    .unwrap_or_else(|e| {
                        panic!("Failed to parse patch file at {patch_file:?}:\n{}", e)
                    })
            };

            // For each patch, map relative paths onto absolute paths, rooted within each's mod directory.
            // We also cache patch targets to short-circuit patching for files that don't need it.
            for patch in &mut patch.patches[..] {
                match patch {
                    Patch::Copy(ref mut x) => {
                        x.sources = x.sources.iter_mut().map(|x| mod_dir.join(x)).collect();
                        targets.insert(x.target.clone());
                    }
                    Patch::Module(ref mut x) => {
                        x.source = mod_dir.join(&x.source);
                        targets.insert(x.before.clone());
                    }
                    Patch::Pattern(x) => {
                        targets.insert(x.target.clone());
                    }
                    Patch::Regex(x) => {
                        targets.insert(x.target.clone());
                    }
                }
            }

            let inner_patches = patch.patches.as_mut(); 
            patches.append(inner_patches);
            var_table.extend(patch.vars);
        }

        PatchTable {
            mod_dir: mod_dir.to_path_buf(),
            loadbuffer: None,
            targets,
            vars: var_table,
            args: HashMap::new(),
            patches,
        }
    }

    /// Set an override for lual_loadbuffer.
    pub fn with_loadbuffer(self, loadbuffer: &'static LoadBuffer) -> Self {
        PatchTable {
            loadbuffer: Some(loadbuffer),
            ..self
        }
    }

    /// Determine if the provided target file / name requires patching.
    pub fn needs_patching(&self, target: &str) -> bool {
        let target = target.strip_prefix('@').unwrap_or(target);
        self.targets.contains(target)
    }

    /// Inject lovely metadata into the game.
    /// # Safety
    /// Unsafe due to internal unchecked usages of raw lua state.
    pub unsafe fn inject_metadata(&self, state: *mut LuaState) {
        let table = vec![
            ("mod_dir", self.mod_dir.to_str().unwrap().replace('\\', "/")),
            ("version", env!("CARGO_PKG_VERSION").to_string()),
        ];

        let mut code = include_str!("../lovely.lua").to_string();
        for (field, value) in table {
            let field = format!("lovely_template:{field}");
            code = code.replace(&field, &value);
        }

        sys::load_module(state, "lovely", &code, self.loadbuffer.as_ref().unwrap())
    }

    /// Apply one or more patches onto the target's buffer.
    /// # Safety
    /// Unsafe due to internal unchecked usages of raw lua state.
    pub unsafe fn apply_patches(&self, target: &str, buffer: &str, lua_state: *mut LuaState) -> String {
        let target = target.strip_prefix('@').unwrap_or(target);

        let module_patches = self
            .patches
            .iter()
            .filter_map(|x| match x {
                Patch::Module(patch) => Some(patch),
                _ => None,
            })
            .collect::<Vec<_>>();
        let copy_patches = self
            .patches
            .iter()
            .filter_map(|x| match x {
                Patch::Copy(patch) => Some(patch),
                _ => None
            })
            .collect::<Vec<_>>();
        let pattern_patches = self
            .patches
            .iter()
            .filter_map(|x| match x {
                Patch::Pattern(patch) => Some(patch),
                _ => None
            })
            .collect::<Vec<_>>();
        let regex_patches = self
            .patches
            .iter()
            .filter_map(|x| match x {
                Patch::Regex(patch) => Some(patch),
                _ => None
            })
            .collect::<Vec<_>>();

        // For display + debug use. Incremented every time a patch is applied.
        let mut patch_count = 0;
        let mut rope = Rope::from_str(buffer);

        // Apply module injection patches.
        let loadbuffer = self.loadbuffer.as_ref().unwrap();
        for patch in module_patches {
            let result = unsafe {
                patch.apply(target, lua_state, &loadbuffer)
            };

            if result {
                patch_count += 1;
            }
        }

        // Apply copy patches.
        for patch in copy_patches {
            if patch.apply(target, &mut rope) {
                patch_count += 1;
            }
        }

        for patch in pattern_patches {
            if patch.apply(target, &mut rope) {
                patch_count += 1;
            }
        }

        for patch in regex_patches {
            if patch.apply(target, &mut rope) {
                patch_count += 1;
            }
        }  

        let mut patched_lines = {
            let inner = rope.to_string();
            inner.split('\n').map(String::from).collect::<Vec<_>>()
        };

        // Apply variable interpolation.
        for line in patched_lines.iter_mut() {
            patch::apply_var_interp(line, &self.vars);
        }

        let patched = patched_lines.join("\n");

        if patch_count == 1 {
            info!("Applied 1 patch to '{target}'");
        } else {
            info!("Applied {patch_count} patches to '{target}'");
        }
        
        // Compute the integrity hash of the patched file.
        let mut hasher = Sha256::new();
        hasher.update(patched.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        format!(
            "LOVELY_INTEGRITY = '{hash}'\n\n{patched}"
        )
    }
}
