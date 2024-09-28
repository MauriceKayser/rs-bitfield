extern crate alloc;

/// [Intel 64 and IA-32 Architectures Software Developer's Manual, Vol. 3A](https://software.intel.com/en-us/articles/intel-sdm)
///
/// # 17.2.3 Debug Status Register (DR6)
///
/// The debug status register (DR6) reports debug conditions that were sampled at the time the last
/// debug exception was generated. Updates to this register only occur when an exception is generated.
///
/// Layout:
///
/// ```
///  63  31      27      23      19      15      11      7       3     0
/// ╔═══╪═══════╧═══════╧═══════╧═════╤═╪═╤═╤═╤═╧═══════╧═══════╪═╤═╤═╤═╗
/// ║   │                             │R│T│S│D│                 │R│R│R│R║
/// ║32x│                             │T│S│S│R│                 │3│2│1│0║ DR6
/// ║ 0 │1 1 1 1 1 1 1 1 1 1 1 1 1 1 1│M│ │ │A│0 1 1 1 1 1 1 1 1│H│H│H│H║
/// ╚═══╧═════════════════════════════╧═╧═╧═╧═╧═════════════════╧═╧═╧═╧═╝
/// R0H = Debug Register 0 Hit
/// R1H = Debug Register 1 Hit
/// R2H = Debug Register 2 Hit
/// R3H = Debug Register 3 Hit
/// DRA = Debug Register Accessed
/// SS  = Single Stepped
/// TS  = Task Switched
/// RTM = Not In Restricted Transactional Memory
/// ```
#[bitfield::bitfield(size)]
#[derive(Debug)]
struct DebugStatus(Status);

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Status {
    DebugRegister0Hit,
    DebugRegister1Hit,
    DebugRegister2Hit,
    DebugRegister3Hit,
    // Bits 4 - 12 are reserved.
    DebugRegisterAccessed = 13,
    SingleStepped,
    TaskSwitched,
    NotInRestrictedTransactionalMemory
    // Bits 17 - 31 are reserved.
}

/// [Intel 64 and IA-32 Architectures Software Developer's Manual](https://software.intel.com/en-us/articles/intel-sdm)
///
/// # 17.2.4 Debug Control Register (DR7)
///
/// The debug control register (DR7) enables or disables breakpoints and sets breakpoint conditions.
///
/// Layout:
///
/// ```
///  63  31      27      23      19      15      11      7       3     0
/// ╔═══╪═══╤═══╪═══╤═══╪═══╤═══╪═══╤═══╪═══╤═╤═╪═╤═╤═╤═╪═╤═╤═╤═╪═╤═╤═╤═╗
/// ║   │Len│Typ│Len│Typ│Len│Typ│Len│Typ│   │D│ │R│ │G│L│G│L│G│L│G│L│G│L║
/// ║32x│ 3 │ 3 │ 2 │ 2 │ 1 │ 1 │ 0 │ 0 │   │R│ │T│ │E│E│3│3│2│2│1│1│0│0║ DR7
/// ║ 0 │   │   │   │   │   │   │   │   │0 0│A│0│M│1│I│I│ │ │ │ │ │ │ │ ║
/// ╚═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╝
/// L0   = Debug Register 0 Is Local
/// G0   = Debug Register 0 Is Global
/// L1   = Debug Register 1 Is Local
/// G1   = Debug Register 1 Is Global
/// L2   = Debug Register 2 Is Local
/// G2   = Debug Register 2 Is Global
/// L3   = Debug Register 3 Is Local
/// G3   = Debug Register 3 Is Global
/// LEI  = Local Is Exact Instruction (legacy)
/// GEI  = Global Is Exact Instruction (legacy)
/// RTM  = Restricted Transactional Memory
/// DRA  = Break On Debug Register Access
/// Typ0 = Debug Register 0 Type
/// Len0 = Debug Register 0 Length
/// Typ1 = Debug Register 1 Type
/// Len1 = Debug Register 1 Length
/// Typ2 = Debug Register 2 Type
/// Len2 = Debug Register 2 Length
/// Typ3 = Debug Register 3 Type
/// Len3 = Debug Register 3 Length
/// ```
#[bitfield::bitfield(size)]
#[derive(Debug)]
struct DebugControl {
    flag: Control,
    #[field(16, 2, complete)] type0: BreakPointType,
    #[field(size = 2)] length0: BreakPointLength,
    #[field(size = 2, complete)] type1: BreakPointType,
    #[field(size = 2)] length1: BreakPointLength,
    #[field(size = 2, complete)] type2: BreakPointType,
    #[field(size = 2)] length2: BreakPointLength,
    #[field(size = 2, complete)] type3: BreakPointType,
    #[field(size = 2)] length3: BreakPointLength
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Control {
    DebugRegister0Local,
    DebugRegister0Global,
    DebugRegister1Local,
    DebugRegister1Global,
    DebugRegister2Local,
    DebugRegister2Global,
    DebugRegister3Local,
    DebugRegister3Global,
    ExactInstructionLocal,
    ExactInstructionGlobal,
    // Bit 10 is reserved.
    RestrictedTransactionalMemory = 11,
    // Bit 12 is reserved.
    DebugRegisterAccess = 13
    // Bits 14 - 15 are reserved.
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum BreakPointType {
    Execute,
    Write,
    ReadWriteIo,
    ReadWrite
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum BreakPointLength {
    One,
    Two,
    #[cfg(target_arch = "x86_64")]
    Eight,
    Four = 3
}

fn main() {
    // Set a break point:
    let control = DebugControl::new() // = GetDR7();
        .set_type0(BreakPointType::Execute) // Not possible as `+ BreakPointType::Execute`, because `BreakPointType` is used more than once.
        .set_length0(BreakPointLength::One) // Not possible as `+ BreakPointLength::One`, because `BreakPointLength` is used more than once.
        + Control::ExactInstructionLocal // Same as: `.set_flag(Control::ExactInstructionLocal, true)`
        + Control::DebugRegister0Local;  // Same as: `.set_flag(Control::DebugRegister0Local, true)`

    println!("{:#?}", &control);

    // SetDR0(break_point_address);
    // SetDR7(control).

    // Handle break point condition:
    let status = DebugStatus::new(); // = GetDR6();
    if status.has(Status::DebugRegister0Hit) {
        // Handle break point.
    }
}