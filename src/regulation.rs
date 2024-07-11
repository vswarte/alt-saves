use std::mem::transmute;
use retour::static_detour;

use crate::match_instruction_pattern;

static_detour! {
    static REGULATIONMANAGER_CONSTRUCTOR: unsafe extern "system" fn(u64, u64) -> u64;
}

const REGULATIONMANAGER_CONSTRUCTOR_PATTERN: &str = concat!(
    // MOV qword ptr [RPS+0x8], RCX
    "01001... 10001001 01001100 ..100100 00001000",
    // PUSH RBX
    "01010011",
    // PUSH RSI
    "01010110",
    // PUSH RDI
    "01010111",
    // SUB RSP, 0x30
    "01001... 10000011 11101100 00110000",
    // MOV qword ptr [RSP+0x20], -0x2
    "01001... 11000111 01000100 ..100100 00100000 11111110 11111111 11111111 11111111",
    // MOV RBX, RCX
    "01001... 10001011 11011001",
    // LEA RAX, <CSRegulationManagerImp::vftable>
    "01001... 10001101 00000101 ........ ........ ........ ........",
    // MOV qword ptr [RCX], RAX
    "01001... 10001001 00000001",
    // MOV qword ptr [RCX+0x8], RDX
    "01001... 10001001 01010001 00001000",
    // LEA RDI, [RCX+0x10]
    "01001... 10001101 01111001 00010000",
    // MOV qword ptr [RSP+0x60], RDI
    "01001... 10001001 01111100 ..100100 01100000",
);

const REGBIN_CHECK_FLAG_SETTER_PATTERN: &str = concat!(
    // MOV RAX, qword ptr [RBX+0x8]
    "01001... 10001011 01000011 00001000",
    // MOV [RAX+0xC8], RCX
    "01001... 10001001 10001000 11001000 00000000 00000000 00000000",
    // CMP [???], CL
    "00111000 00001101 ........ ........ ........ ........",
    // JNZ [???]
    "01110101 ........",
    // CALL [???]
    "11101000 ........ ........ ........ ........",
    // MOV [RegBinFlags + 0], AL
    "10001000 00000101 ........ ........ ........ ........",
    // MOV [RegBinFlags + 1], AL
    "10001000 00000101 ........ ........ ........ ........",
    // MOV [RegBinFlags + 2], AL
    "10001000 00000101 [........ ........ ........ ........]",
);

pub fn hook() {
    // Find the constructor's pointer
    let regulationmanager_constructor = match_instruction_pattern(REGULATIONMANAGER_CONSTRUCTOR_PATTERN)
        .expect("Could not find regulation manager constructor")
        .location;

    // Find the regbin check flag by matching the second IBO and checking what offsets the code
    // references.
    let regbin_check_flag = {
        let matched = match_instruction_pattern(REGBIN_CHECK_FLAG_SETTER_PATTERN)
            .expect("Could not find the regbin check flag setter");

        let capture = matched.captures.first().unwrap();
        let offset = u32::from_le_bytes(capture.bytes.as_slice().try_into().unwrap());

        // We take the occurence location, add the offset from the bytes and add the size of the
        // instruction itself (minus capture group offset in instruction) to find the
        // absolute position.
        capture.location + offset as usize + 4
    };

    unsafe {
        REGULATIONMANAGER_CONSTRUCTOR
            .initialize(
                transmute(regulationmanager_constructor),
                move |allocated_space: u64, param_2: u64| {
                    let result = REGULATIONMANAGER_CONSTRUCTOR.call(allocated_space, param_2);
                    // Overwrites the flag that seems to determine if the regulation
                    // bin file should be checked against a particular hash in the sl2.
                    *(regbin_check_flag as *mut u8) = 0x0;

                    result
                }
            )
            .unwrap();

        REGULATIONMANAGER_CONSTRUCTOR.enable().unwrap();
    }
}
