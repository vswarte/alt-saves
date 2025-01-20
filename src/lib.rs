mod regulation;

use std::error::Error;
use std::fs::read_to_string;
use std::mem::transmute;
use std::path;

use retour::static_detour;
use serde::Deserialize;
use tracing_panic::panic_hook;
use windows::core::{s, HSTRING, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

static_detour! {
    static CREATE_FILE_W_HOOK: unsafe extern "C" fn(PCWSTR, u32, u32, u64, u32, u32, HANDLE) -> u64;
}

#[no_mangle]
pub unsafe extern "C" fn DllMain(_base: usize, reason: u32) -> bool {
    match reason {
        1 => {
            std::panic::set_hook(Box::new(panic_hook));
            let appender = tracing_appender::rolling::never("./", "altsaves.log");
            tracing_subscriber::fmt().with_writer(appender).init();

            let config = Config::from_file(path::Path::new("./altsaves.toml")).unwrap_or_default();
            init(config).expect("Could not initialize altsaves.");
        }
        _ => {}
    }

    true
}

pub fn init(config: Config) -> Result<(), Box<dyn Error>> {
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

                    CREATE_FILE_W_HOOK.call(
                        path,
                        access,
                        share,
                        security,
                        disposition,
                        attributes,
                        template,
                    )
                },
            )?
            .enable()?;
    }

    tracing::info!("CreateFileW {create_file_w:x}");

    regulation::hook();

    Ok(())
}

#[derive(Deserialize)]
pub struct Config {
    pub extension: String,
}

impl Config {
    pub fn from_file(path: &path::Path) -> Result<Self, Box<dyn Error>> {
        let config = read_to_string(path)?;
        Ok(toml::from_str(&config)?)
    }
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
    // fn as_pcwstr(&self) -> PCWSTR {
    //     match self {
    //         RequestedPath::Untouched(p) => *p,
    //         RequestedPath::Rewritten(s) => {
    //             todo!()
    //         }
    //     }
    // }
}
