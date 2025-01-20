use pelite::pattern;
use pelite::pattern::Atom;
use pelite::pe::{Pe, PeView};
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Memory::VirtualProtect;
use windows::Win32::System::Memory::{PAGE_PROTECTION_FLAGS, PAGE_READWRITE};

const REGBIN_SAFETY_CHECK_PATCH: &[Atom] = pattern!(
    "
    48 8b 43 08
    48 89 88 c8 00 00 00
    38 0d ? ? ? ?
    75 ?
    e8 ? ? ? ?
    88 05 ? ? ? ?
    88 05 ? ? ? ?
    ' 88 05 ? ? ? ?
    "
);

pub fn hook() {
    let mut matches = [0; 2];
    let view = unsafe {
        PeView::module(GetModuleHandleA(PCSTR(std::ptr::null())).unwrap().0 as *const u8)
    };
    if !view
        .scanner()
        .finds_code(REGBIN_SAFETY_CHECK_PATCH, &mut matches)
    {
        panic!("Failed to find the pattern for regbin safety check");
    }
    let addr = view
        .rva_to_va(matches[1])
        .expect("Failed to convert rva to va") as *mut u8;

    unsafe {
        let mut old_protect = PAGE_PROTECTION_FLAGS::default();
        VirtualProtect(addr as _, 1, PAGE_READWRITE, &mut old_protect).unwrap();

        // XOR instead of MOV so we clear out the flag
        std::ptr::write(addr, 0x30);

        VirtualProtect(addr as _, 1, old_protect, &mut old_protect).unwrap();
    }
}
