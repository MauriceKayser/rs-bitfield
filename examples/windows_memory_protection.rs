#![allow(unused)]

extern crate alloc;

/// This could also be solved by using a generic bit field for `FlagAlloc` and `FlagProtect`.
///
/// [Microsoft Docs: Memory Protection Constants](https://docs.microsoft.com/en-us/windows/desktop/Memory/memory-protection-constants)
///
/// [GitHub: ProcessHacker](https://github.com/processhacker/processhacker/blob/master/phnt/include/ntmmapi.h)
///
/// Stores the protection options for virtual memory, which can only be assigned to a whole page.
///
/// Layout:
///
/// ```
///  31      27      23      19      15      11      7       3     0
/// ╔═╤═╤═╤═╧═══════╧═══════╧═══════╧═══════╧═╤═╤═╤═╪═══════╧═══════╗
/// ║E│T│E│                                   │W│N│G│Access         ║
/// ║T│N│U│                                   │C│C│ │               ║
/// ║C│U│ │0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0│ │ │ │               ║
/// ╟─┼─┼─┴───────────────────────────────────┴─┴─┴─┴───────────────╢ Protection
/// ║R│T│                                                           ║
/// ║F│I│                                                           ║
/// ║M│ │                                                           ║
/// ╚═╧═╧═══════════════════════════════════════════════════════════╝
///       Access
/// G   = Guard
/// NC  = No Cache
/// WC  = Write Combine
/// EU  = Enclave Unvalidated
/// TNU = Targets No Update
/// ETC = Enclave Thread Control
/// TI  = Targets Invalid
/// RFM = Revert to File Map
/// ```
#[bitfield::bitfield(32, allow_overlaps)]
#[derive(Debug)]
struct Protection {
    #[field(size = 8)]
    access: Access,
    flag: Flag,
    flag_alloc: FlagAlloc,
    flag_enclave: FlagEnclave,
    flag_protect: FlagProtect,
    flag_unknown: FlagUnknown
}

// Getters for specific access flags.
impl Protection {
    // TODO: Add `const` when https://github.com/rust-lang/rfcs/pull/2632 is merged.
    fn copy_on_write(&self) -> bool {
        match self.access() {
            Ok(Access::ReadWriteCopy) |
            Ok(Access::ExecuteReadWriteCopy) => true,
            _ => false
        }
    }

    // TODO: Add `const` when https://github.com/rust-lang/rfcs/pull/2632 is merged.
    fn execute(&self) -> bool {
        match self.access() {
            Ok(Access::Execute) |
            Ok(Access::ExecuteRead) |
            Ok(Access::ExecuteReadWrite) |
            Ok(Access::ExecuteReadWriteCopy) => true,
            _ => false
        }
    }

    // TODO: Add `const` when https://github.com/rust-lang/rfcs/pull/2632 is merged.
    fn read(&self) -> bool {
        match self.access() {
            Ok(Access::Read) |
            Ok(Access::ReadWrite) |
            Ok(Access::ReadWriteCopy) |
            Ok(Access::ExecuteRead) |
            Ok(Access::ExecuteReadWrite) |
            Ok(Access::ExecuteReadWriteCopy) => true,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Ok(Access::Execute) => true,
            _ => false
        }
    }

    // TODO: Add `const` when https://github.com/rust-lang/rfcs/pull/2632 is merged.
    fn write(&self) -> bool {
        match self.access() {
            Ok(Access::ReadWrite) |
            Ok(Access::ReadWriteCopy) |
            Ok(Access::ExecuteReadWrite) |
            Ok(Access::ExecuteReadWriteCopy) => true,
            _ => false
        }
    }
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum Access {
    // 0 is reserved.
    None = 1 << 0,
    Read = 1 << 1,
    ReadWrite = 1 << 2,
    ReadWriteCopy = 1 << 3,
    Execute = 1 << 4,
    ExecuteRead = 1 << 5,
    ExecuteReadWrite = 1 << 6,
    ExecuteReadWriteCopy = 1 << 7
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Flag {
    Guard = 9,
    NoCache,
    WriteCombine
    // Bits 11 - 28 are reserved.
}

/// To be used by `VirtualAlloc`.
#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum FlagAlloc {
    TargetsInvalid = 30
}

/// To be used by `LoadEnclaveData`.
#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum FlagEnclave {
    Unvalidated = 29,
    ThreadControl = 31
}

/// To be used by `VirtualProtect`.
#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum FlagProtect {
    TargetsNoUpdate = 30
}

/// Usage unknown.
#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum FlagUnknown {
    RevertToFileMap = 31
}

fn main() {
    let protection = Protection::new()
        + Access::ExecuteReadWrite   // Same as: `.set_access(Access::ExecuteReadWrite)`
        + Flag::Guard                // Same as: `.set_flag(Flag::Guard, true)`
        + Flag::NoCache              // Same as: `.set_flag(Flag::NoCache, true)`
        + FlagAlloc::TargetsInvalid; // Same as: `.set_flag_alloc(FlagAlloc::TargetsInvalid, true)`

    assert_eq!(
        protection.0,

        (1 <<  6) + // access
        (1 <<  9) + // Flag::Guard
        (1 << 10) + // Flag::NoCache
        (1 << 30)   // FlagAlloc::TargetsInvalid / FlagProtect::TargetsNoUpdate
    );

    assert_eq!(&format!("{:#?}", &protection),
"Protection {
    access: ExecuteReadWrite,
    flag: Flag {
        Guard: true,
        NoCache: true,
        WriteCombine: false,
    },
    flag_alloc: FlagAlloc {
        TargetsInvalid: true,
    },
    flag_enclave: FlagEnclave {
        Unvalidated: false,
        ThreadControl: false,
    },
    flag_protect: FlagProtect {
        TargetsNoUpdate: true,
    },
    flag_unknown: FlagUnknown {
        RevertToFileMap: false,
    },
}"
    );

    println!("{:#?}", &protection);
}