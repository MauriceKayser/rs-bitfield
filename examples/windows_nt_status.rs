extern crate alloc;

/// [Microsoft Docs: NTSTATUS](https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-erref/87fba13e-bf06-450e-83b1-9241dc81e781)
///
/// System-supplied status codes.
///
/// Layout:
///
/// ```
///  31      27      23      19      15      11      7       3     0
/// ╔═══╤═╤═╪═══════╧═══════╧═══════╪═══════╧═══════╧═══════╧═══════╗
/// ║Sev│C│R│Facility               │Code                           ║
/// ║   │ │H│                       │                               ║ NtStatus
/// ║   │ │R│                       │                               ║
/// ╚═══╧═╧═╧═══════════════════════╧═══════════════════════════════╝
///       Code
///       Facility
/// RHR = Reserved for HRESULT
/// C   = Customer
/// Sev = Severity
/// ```
#[bitfield::bitfield(NonZero32)]
#[derive(Debug, Eq, PartialEq)]
struct NtStatus {
    code: u16,
    #[field(size = 12)]
    facility: Facility,
    reserved: bool,
    customer: bool,
    #[field(size = 2)]
    severity: Severity
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u16)]
enum Facility {
    None,
    Debugger,
    RemoteProcedureCallRuntime,
    RemoteProcedureCallStubs,
    Io,
    // Variants 5 - 6 are reserved.
    NtWin32 = 7,
    // Variant 8 is reserved.
    NtSecuritySupportProviderInterface = 9,
    TerminalServer,
    MultilingualUserInterface,
    // Variants 12 - 15 are reserved.
    UniversalSerialBus = 16,
    HumanInterfaceDevices,
    FireWire,
    Cluster,
    AdvancedConfigurationAndPowerInterface,
    SideBySide,
    // Variants 22 - 24 are reserved.
    Transaction = 25,
    CommonLog,
    Video,
    FilterManager,
    Monitor,
    GraphicsKernel,
    // Variant 31 is reserved.
    DriverFramework = 32,
    FullVolumeEncryption,
    FilterPlatform,
    NetworkDriverInterface,
    // Variants 36 - 52 are reserved.
    Hypervisor = 53,
    IpSec
    // Variants 55 - 0xFFF are reserved.
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum Severity {
    Success,
    Information,
    Warning,
    Error
}

/// The NTSTATUS value `0` indicates success. That is why an error can be represented as
/// `Option<NtStatus>`, where `None` is mapped to `0` because of the `NonZero32` bitfield type.
#[allow(non_snake_case)]
fn NtTestFunction(return_something: bool) -> Option<NtStatus> {
    return_something.then(||
        NtStatus(unsafe { *(&0u32 as *const u32 as *const _) })
        .set_code(5).unwrap()
        .set_severity(Severity::Error).unwrap()
    )
}

fn main() {
    assert_eq!(core::mem::size_of::<NtStatus>(), 4);
    assert_eq!(core::mem::size_of::<Option<NtStatus>>(), 4);
    assert_eq!(NtTestFunction(false), None);
    assert_eq!(NtTestFunction(true).map(|s| s.0.get()), Some(0xC0000005));
}