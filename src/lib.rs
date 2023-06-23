#![feature(absolute_path)]

use std::mem;

mod config;

use broadsword::dll;
use broadsword::runtime;
use detour::static_detour;

use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use crate::config::{get_rewrite_extension, get_seamless_rewrite_extension};

const SAVEGAME_EXTENSION: &str = ".sl2";
const SAVEGAME_BACKUP_EXTENSION: &str = ".sl2.bak";
const SC_SAVEGAME_EXTENSION: &str = ".co2";
const SC_SAVEGAME_BACKUP_EXTENSION: &str = ".co2.bak";

static_detour! {
    static CREATE_FILE_W_HOOK: unsafe extern "system" fn(PCWSTR, u32, u32, u64, u32, u32, HANDLE) -> u64;
}

#[dll::entrypoint]
pub fn entry(_: usize) -> bool {
    broadsword::logging::init("log/alt-saves.log");
    apply_hooks();
    return true;
}

fn apply_hooks() {
    let create_file_w = runtime::get_module_symbol("kernel32", "CreateFileW")
        .expect("Could not find CreateFileW");

    unsafe {
        CREATE_FILE_W_HOOK
            .initialize(
                mem::transmute(create_file_w),
                move |path: PCWSTR,
                      desired_access: u32,
                      share_mode: u32,
                      security_attributes: u64,
                      creation_disposition: u32,
                      flags_and_attributes: u32,
                      template_file: HANDLE| {

                    // Doing this here to ensure the string isn't dropped until after the fn call
                    // otherwise the string's source is dropped before the pointer is consumed.
                    let patched_path = transform_path(path)
                        .map(|s| HSTRING::from(s));

                    let effective_path = match patched_path {
                        None => path,
                        Some(s) => PCWSTR::from_raw(s.as_ptr()),
                    };

                    CREATE_FILE_W_HOOK.call(
                        effective_path,
                        desired_access,
                        share_mode,
                        security_attributes,
                        creation_disposition,
                        flags_and_attributes,
                        template_file,
                    )
                },
            )
            .unwrap();

        CREATE_FILE_W_HOOK.enable().unwrap();
    }
}

// TODO: Rewrites can be cached but is it worth the performance gain with how little it's called?
unsafe fn transform_path(path: PCWSTR) -> Option<String> {
    let path_string = path.to_string()
        .expect("Could not convert PCWSTR into string");

    if path_string.ends_with(SAVEGAME_EXTENSION) || path_string.ends_with(SAVEGAME_BACKUP_EXTENSION) {
        Some(path_string.clone().replace(
            SAVEGAME_EXTENSION,
            get_rewrite_extension().as_str(),
        ))
    } else if path_string.ends_with(SC_SAVEGAME_EXTENSION) || path_string.ends_with(SC_SAVEGAME_BACKUP_EXTENSION) {
        let extension = get_seamless_rewrite_extension()
            .unwrap_or(get_rewrite_extension());

        Some(path_string.clone().replace(
            SC_SAVEGAME_EXTENSION,
            extension.as_str(),
        ))
    } else {
        None
    }
}