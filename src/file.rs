use crate::config::{get_rewrite_extension, get_seamless_rewrite_extension};

use std::mem::transmute;

use broadsword::runtime;
use retour::static_detour;
use windows::{core::{HSTRING, PCWSTR}, Win32::Foundation::HANDLE};

const SAVEGAME_EXTENSION: &str = ".sl2";
const SAVEGAME_BACKUP_EXTENSION: &str = ".sl2.bak";
const SC_SAVEGAME_EXTENSION: &str = ".co2";
const SC_SAVEGAME_BACKUP_EXTENSION: &str = ".co2.bak";

static_detour! {
    static CREATE_FILE_W_HOOK: unsafe extern "C" fn(PCWSTR, u32, u32, u64, u32, u32, HANDLE) -> u64;
}

pub fn hook() {
    // Hook Kernel32's CreateFileW since that is responsible for opening file
    // handles, and it's nice and documented.
    let create_file_w = runtime::get_module_symbol("kernel32", "CreateFileW")
        .expect("Could not find CreateFileW");

    unsafe {
        CREATE_FILE_W_HOOK
            .initialize(
                transmute(create_file_w),
                create_file_hook,
            )
            .unwrap();

        CREATE_FILE_W_HOOK.enable().unwrap();
    }
}

/// Handle an actual invoke of CreateFileW and do necessary rewrites.
fn create_file_hook(
    path: PCWSTR,
    desired_access: u32,
    share_mode: u32,
    security_attributes: u64,
    creation_disposition: u32,
    flags_and_attributes: u32,
    template_file: HANDLE,
) -> u64 {
    let patched_path = unsafe { transform_path(path) }
        .map(HSTRING::from);

    let result_path = match patched_path {
        None => path,
        Some(s) => PCWSTR::from_raw(s.as_ptr()),
    };

    unsafe {
        CREATE_FILE_W_HOOK.call(
            result_path,
            desired_access,
            share_mode,
            security_attributes,
            creation_disposition,
            flags_and_attributes,
            template_file,
        )
    }
}

/// Transforms the input path for CreateFileW, yields Some() with a new path or
/// None if no rewriting was necessary.
unsafe fn transform_path(path: PCWSTR) -> Option<String> {
    // TODO: this logic can be broken up for readability
    let path_string = path.to_string()
        .expect("Could not convert PCWSTR into string");

    if path_string.ends_with(SAVEGAME_EXTENSION) ||
        path_string.ends_with(SAVEGAME_BACKUP_EXTENSION) {

        Some(path_string.replace(SAVEGAME_EXTENSION, get_rewrite_extension()))
    } else if path_string.ends_with(SC_SAVEGAME_EXTENSION) ||
        path_string.ends_with(SC_SAVEGAME_BACKUP_EXTENSION) {

        let extension = get_seamless_rewrite_extension()
            .map(|f| f.as_str())
            .unwrap_or(get_rewrite_extension());

        Some(path_string.replace(SC_SAVEGAME_EXTENSION, extension))
    } else {
        None
    }
}
