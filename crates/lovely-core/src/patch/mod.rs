use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use anyhow::{ensure, Result, Context};

pub use copy::CopyPatch;
pub use module::ModulePatch;
pub use pattern::PatternPatch;
pub use regex::RegexPatch;

pub mod copy;
pub mod loader;
pub mod module;
pub mod pattern;
pub mod regex;
pub mod table;
pub mod vars;

pub type Priority = i32;

#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub version: String,
    // Does nothing, kept for legacy compat
    #[serde(default)]
    pub dump_lua: bool,
    #[serde(default)]
    pub priority: Priority,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModMetadata {
    pub id: String
}

impl ModMetadata {
    pub fn validate(&self) -> Result<()> {
        self.validate_id().with_context(|| format!("Invalid ID: {:?}", self.id))?;
        return Ok(());
    }

    pub fn validate_id(&self) -> Result<()> {
        let len = self.id.len();
        ensure!(len <= 128, "Metadata ID must be less than 128 characters in length");
        ensure!(len >= 2, "Metadata ID must be more than 2 characters in length");
        ensure!(self.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'), "Mod ID can only contain a-z A-Z 0-9 _ characters");
        return Ok(());
    }
}

// Represents a single .toml file after deserialization.
#[derive(Serialize, Deserialize, Debug)]
pub struct PatchFile {
    pub manifest: Manifest,
    pub patches: Vec<Patch>,
    // A table of info about a mod
    // limit one per set of patches in a mod.
    #[serde(default)]
    pub metadata: Option<ModMetadata>,

    // A table of variable name = value bindings. These are interpolated
    // into injected source code as the *last* step in the patching process.
    #[serde(default)]
    pub vars: HashMap<String, String>,
    // // A table of arguments, read and parsed from the environment command line.
    // // Binds double-hyphenated argument names (--arg) to a value, with additional metadata
    // // available to produce help messages, set default values, and apply other behavior.
    // #[serde(default)]
    // pub args: HashMap<String, PatchArgs>,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct PatchArgs {
//     // An optional help string. This will be printed out in the calling console
//     // (if available) when the --help argument is supplied.
//     pub help: Option<String>,

//     // An optional default value. Not including a default value will cause Lovely
//     // to panic if this argument is missing or could not be parsed.
//     // Consider this to be both a "default value" and a "required" field, depending
//     // on whether or not it's set.
//     pub default: Option<String>,

//     // This field allows for a patch author to force lovely to parse incoming arguments
//     // with the exact name that they are defined by.
//     // This disables lovely's automatic underscore to hyphen conversion.
//     #[serde(default)]
//     pub name_override: bool,

//     // This field allows for arguments (--arg) to be passed without implicit values,
//     // treating it essentially as a flag. If it exists in the args, it's true, if not,
//     // then we set it to false.
//     #[serde(default)]
//     pub treat_as_flag: bool,
// }

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Patch {
    // A patch which applies some change to a series of line(s) after a line with a match
    // to the provided pattern has been found.
    Pattern(PatternPatch),
    Regex(RegexPatch),
    Copy(CopyPatch),
    Module(ModulePatch),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Target {
    Single(String),
    Multi(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum InsertPosition {
    At,
    Before,
    After,
}
