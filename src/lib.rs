use std::error::Error;
use std::ffi::CString;
use std::fs::read_to_string;
use std::mem::transmute;
use std::path;

use pelite::pattern;
use pelite::pattern::Atom;
use pelite::pe::{imports::Import, Pe, PeFile, PeObject, PeView};
use pelite::util::CStr;
use retour::static_detour;
use serde::Deserialize;
use tracing_panic::panic_hook;
use windows::core::{s, HSTRING, PCSTR, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

static_detour! {
    static CREATE_FILE_W_HOOK: unsafe extern "C" fn(PCWSTR, u32, u32, u64, u32, u32, HANDLE) -> u64;
}

const REGBIN_SAFETY_CHECK_PATCH: &[Atom] = pattern!("");

#[no_mangle]
pub unsafe extern "C" fn DllMain(_base: usize, reason: u32) -> bool {
    match reason {
        1 => {
            std::panic::set_hook(Box::new(panic_hook));
            let appender = tracing_appender::rolling::never("./", "altsaves.log");
            tracing_subscriber::fmt().with_writer(appender).init();

            init().expect("Could not initialize altsaves.");
        }
        _ => {}
    }

    true
}

fn init() -> Result<(), Box<dyn Error>> {
    // Read config from file or get default
    let config: Config = path::absolute(path::PathBuf::from("./altsaves.toml"))
        .ok()
        .and_then(|p| read_to_string(p).ok())
        .and_then(|c| toml::from_str(&c).ok())
        .unwrap_or_default();

    // Find CreateFileW. For some reason I couldn't use pelite to parse the IAT so :shrug:
    let create_file_w = unsafe {
        let kernel32 = GetModuleHandleA(s!("kernel32"))?;
        GetProcAddress(kernel32, s!("CreateFileW")).expect("Could find CreateFileW")
    } as usize;

    // Hook the CreateFileW so we can swap out paths where required
    unsafe {
        CREATE_FILE_W_HOOK
            .initialize(
                transmute(create_file_w),
                move |path, access, share, security, disposition, attributes, template| {
                    let result_path = if path.as_wide().ends_with(&[0x2E, 0x73, 0x6C, 0x32]) {
                        let path = path.to_string().unwrap();
                        RequestedPath::Rewritten(HSTRING::from(format!(
                            "{}{}",
                            &path[..path.len() - 4],
                            &config.extension
                        )))
                    } else {
                        RequestedPath::Untouched(path)
                    };

                    tracing::info!("Rewritten {:?}", result_path);

                    unsafe {
                        CREATE_FILE_W_HOOK.call(
                            path,
                            access,
                            share,
                            security,
                            disposition,
                            attributes,
                            template,
                        )
                    }
                },
            )?
            .enable()?;
    }

    tracing::info!("CreateFileW {create_file_w:x}");
    Ok(())
}

#[derive(Deserialize)]
pub struct Config {
    pub extension: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            extension: ".mod".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum RequestedPath {
    Untouched(PCWSTR),
    Rewritten(HSTRING),
}

impl RequestedPath {
    fn as_pcwstr(&self) -> PCWSTR {
        match self {
            RequestedPath::Untouched(p) => *p,
            RequestedPath::Rewritten(s) => {
                todo!()
            }
        }
    }
}

// #[dll::entrypoint]
// pub fn entry(_: usize) -> bool {
//     broadsword::logging::init("alt-saves.log");
//
//     // file::hook();
//     // regulation::hook();
//     true
// }

// #[derive(Deserialize)]
// pub struct Config {
//     pub extension: String,
// }
//
// /// Takes an instruction pattern and looks for its location
// pub fn match_instruction_pattern(pattern: &str) -> Option<PatternResult> {
//     // Find .text section details since that's where the code lives
//     let text_section = runtime::get_module_section_range("eldenring.exe", ".text")
//         .or_else(|_| runtime::get_module_section_range("start_protected_game.exe", ".text"))
//         .unwrap();
//
//     // Represent search area as a slice
//     let scan_slice = unsafe {
//         slice::from_raw_parts(
//             text_section.start as *const u8,
//             text_section.end - text_section.start,
//         )
//     };
//
//     let pattern = scanner::Pattern::from_bit_pattern(pattern).unwrap();
//     scanner::simple::scan(scan_slice, &pattern)
//         // TODO: this kinda of rebasing can be done in broadsword probably
//         .map(|result| PatternResult {
//             location: text_section.start + result.location,
//             captures: result.captures.into_iter()
//                 .map(|capture| {
//                     PatternCapture {
//                         location: text_section.start + capture.location,
//                         bytes: capture.bytes,
//                     }
//                 })
//                 .collect()
//         })
// }
//
// #[derive(Debug)]
// pub struct PatternResult {
//     location: usize,
//     captures: Vec<PatternCapture>,
// }
//
// #[derive(Debug)]
// pub struct PatternCapture {
//     location: usize,
//     bytes: Vec<u8>,
// }
