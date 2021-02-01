extern crate alloc;

/// [Microsoft Docs: Access Mask Format](https://docs.microsoft.com/en-us/windows/desktop/secauthz/access-mask-format)
///
/// An access right is a bit flag that corresponds to a particular set of operations that a thread
/// can perform on a securable object. If a thread tries to perform an operation on an object, but
/// does not have the necessary access right to the object, the system does not carry out the operation.
///
/// Layout:
///
/// ```
///  31      27      23      19      15      11      7       3     0
/// ╔═╤═╤═╤═╪═══╤═╤═╪═════╤═╪═╤═╤═╤═╪═══════╧═══════╧═══════╧═══════╗
/// ║G│G│G│G│   │M│S│     │S│W│W│R│D│Object-specific                ║
/// ║R│W│E│A│   │A│S│     │ │O│D│C│ │                               ║ Access
/// ║ │ │ │ │0 0│ │ │0 0 0│ │ │ │ │ │                               ║
/// ╚═╧═╧═╧═╧═══╧═╧═╧═════╧═╧═╧═╧═╧═╧═══════════════════════════════╝
///      Object-specific
/// D  = Delete
/// RC = Read Control
/// WD = Write DAC
/// WO = Write Owner
/// S  = Synchronize
/// SS = System Security
/// MA = Maximum Allowed
/// GA = Generic All
/// GE = Generic Execute
/// GW = Generic Write
/// GR = Generic Read
/// ```
#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Standard {
    Delete = 16,
    ReadControl,
    WriteDac,
    WriteOwner,
    Synchronize,
    // Bits 21 - 23 are reserved.
    SystemSecurity = 24,
    MaximumAllowed,
    // Bits 26 and 27 are reserved.
    GenericAll = 28,
    GenericExecute,
    GenericWrite,
    GenericRead
}

/// [Microsoft Docs: File Access Rights Constants](https://docs.microsoft.com/en-us/windows/desktop/FileIO/file-access-rights-constants)
///
/// Layout:
///
/// ```
///  15      11      7       3     0
/// ╔═══════╧═════╤═╪═╤═╤═╤═╪═╤═╤═╤═╗
/// ║             │W│R│D│T│W│R│A│A│L║
/// ║             │A│A│C│ │E│E│S│F│ ║ Directory
/// ║0 0 0 0 0 0 0│ │ │ │ │A│A│D│ │ ║
/// ╚═════════════╧═╧═╧═╧═╧═╧═╧═╧═╧═╝
/// L   = List
/// AF  = Add File
/// ASD = Add Sub Directory
/// REA = Read Extended Attributes
/// WEA = Write Extended Attributes
/// T   = Traverse
/// DC  = Delete Child
/// RA  = Read Attributes
/// WA  = Write Attributes
/// ```
#[bitfield::bitfield(32)]
#[derive(Debug)]
struct AccessDirectory {
    object: Directory,
    standard: Standard
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Directory {
    List,
    AddFile,
    AddSubDirectory,
    ReadExtendedAttributes,
    WriteExtendedAttributes,
    Traverse,
    DeleteChild,
    ReadAttributes,
    WriteAttributes
    // Bits 9 - 15 are reserved.
}

/// [Microsoft Docs: File Access Rights Constants](https://docs.microsoft.com/en-us/windows/desktop/FileIO/file-access-rights-constants)
///
/// Layout:
///
/// ```
///  15      11      7       3     0
/// ╔═══════╧═════╤═╪═╤═╤═╤═╪═╤═╤═╤═╗
/// ║             │W│R│ │E│W│R│A│W│R║
/// ║             │A│A│ │ │E│E│ │ │ ║ File
/// ║0 0 0 0 0 0 0│ │ │0│ │A│A│ │ │ ║
/// ╚═════════════╧═╧═╧═╧═╧═╧═╧═╧═╧═╝
/// R   = Read
/// W   = Write
/// A   = Append
/// E   = Execute
/// REA = Read Extended Attributes
/// WEA = Write Extended Attributes
/// RA  = Read Attributes
/// WA  = Write Attributes
/// ```
#[bitfield::bitfield(32)]
#[derive(Debug)]
struct AccessFile {
    object: File,
    standard: Standard
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum File {
    Read,
    Write,
    Append,
    ReadExtendedAttributes,
    WriteExtendedAttributes,
    Execute,
    // Bit 6 is reserved.
    ReadAttributes = 7,
    WriteAttributes
    // Bits 9 - 15 are reserved.
}

/// [Microsoft Docs: Synchronization Object Security and Access Rights](https://docs.microsoft.com/en-us/windows/win32/sync/synchronization-object-security-and-access-rights)
///
/// Layout:
///
/// ```
///  15      11      7       3     0
/// ╔═══════╧═══════╧═══════╧═══╤═╤═╗
/// ║                           │S│G║
/// ║                           │I│I║ Event
/// ║0 0 0 0 0 0 0 0 0 0 0 0 0 0│ │ ║
/// ╚═══════════════════════════╧═╧═╝
/// GI = Get Information
/// SI = Set Information
/// ```
#[bitfield::bitfield(32)]
#[derive(Debug)]
struct AccessEvent {
    object: Event,
    standard: Standard
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Event {
    GetInformation,
    SetInformation
    // Bits 2 - 15 are reserved.
}

/// [Microsoft Docs: Job Access Rights Constants](https://docs.microsoft.com/en-us/windows/desktop/ProcThread/job-object-security-and-access-rights)
///
/// Layout:
///
/// ```
///  15      11      7       3     0
/// ╔═══════╧═══════╧═════╤═╪═╤═╤═╤═╗
/// ║                     │S│T│G│S│A║
/// ║                     │S│ │I│I│P║ Job
/// ║0 0 0 0 0 0 0 0 0 0 0│I│ │ │ │ ║
/// ╚═════════════════════╧═╧═╧═╧═╧═╝
/// AP  = Assign Process
/// SI  = Set Information
/// GI  = Get Information
/// T   = Terminate
/// SSI = Set Security Information
/// ```
#[bitfield::bitfield(32)]
#[derive(Debug)]
struct AccessJob {
    object: Job,
    standard: Standard
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Job {
    AssignProcess,
    SetInformation,
    GetInformation,
    Terminate,
    SetSecurityInformation
    // Bits 5 - 15 are reserved.
}

/// [Microsoft Docs: File Access Rights Constants](https://docs.microsoft.com/en-us/windows/desktop/FileIO/file-access-rights-constants)
///
/// Layout:
///
/// ```
///  15      11      7       3     0
/// ╔═══════╧═════╤═╪═╤═══╤═╪═╤═╤═══╗
/// ║             │W│R│   │W│R│C│   ║
/// ║             │A│A│   │E│E│ │   ║ Pipe
/// ║0 0 0 0 0 0 0│ │ │0 0│A│A│ │0 0║
/// ╚═════════════╧═╧═╧═══╧═╧═╧═╧═══╝
/// C   = Create
/// REA = Read Extended Attributes
/// WEA = Write Extended Attributes
/// RA  = Read Attributes
/// WA  = Write Attributes
/// ```
#[bitfield::bitfield(32)]
#[derive(Debug)]
struct AccessPipe {
    object: Pipe,
    standard: Standard
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Pipe {
    // Bits 0 - 1 are reserved.
    Create = 2,
    ReadExtendedAttributes,
    WriteExtendedAttributes,
    // Bits 5 - 6 are reserved.
    ReadAttributes = 7,
    WriteAttributes
    // Bits 9 - 15 are reserved.
}

/// [Microsoft Docs: Process Access Rights Constants](https://docs.microsoft.com/en-us/windows/desktop/ProcThread/process-security-and-access-rights)
///
/// Layout:
///
/// ```
///  15      11      7       3     0
/// ╔═════╤═╪═╤═╤═╤═╪═╤═╤═╤═╪═╤═╤═╤═╗
/// ║     │G│S│G│S│S│C│D│V│V│V│S│C│T║
/// ║     │L│R│I│I│Q│P│H│M│M│M│S│T│ ║ Process
/// ║0 0 0│I│ │ │ │ │ │ │W│R│O│I│ │ ║
/// ╚═════╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╝
/// T   = Terminate
/// CT  = Create Thread
/// SSI = Set Session Id
/// VMO = Virtual Memory Operation
/// VMR = Virtual Memory Read
/// VMW = Virtual Memory Write
/// DH  = Duplicate Handle
/// CP  = Create Process
/// SQ  = Set Quota
/// SI  = Set Information
/// GI  = Get Information
/// SR  = Suspend + Resume
/// GLI = Get Limited Information
/// ```
#[bitfield::bitfield(32)]
#[derive(Debug)]
struct AccessProcess {
    object: Process,
    standard: Standard
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Process {
    Terminate,
    CreateThread,
    SetSessionId,
    VirtualMemoryOperation,
    VirtualMemoryRead,
    VirtualMemoryWrite,
    DuplicateHandle,
    CreateProcess,
    SetQuota,
    SetInformation,
    GetInformation,
    SuspendResume,
    GetLimitedInformation
    // Bits 13 - 15 are reserved.
}

/// [Microsoft Docs: Registry Access Rights Constants](https://docs.microsoft.com/en-us/windows/desktop/SysInfo/registry-key-security-and-access-rights)
///
/// Layout:
///
/// ```
///  15      11      7       3     0
/// ╔═══════╧═══╤═╤═╪═══╤═╤═╪═╤═╤═╤═╗
/// ║           │W│W│   │C│N│L│C│S│G║
/// ║           │3│6│   │L│ │S│S│V│V║ Registry
/// ║0 0 0 0 0 0│2│4│0 0│ │ │K│K│ │ ║
/// ╚═══════════╧═╧═╧═══╧═╧═╧═╧═╧═╧═╝
/// GV  = Get Value
/// SV  = Set Value
/// CSK = Create Sub Key
/// LSK = List Sub Keys
/// N   = Notify
/// CL  = Create Link
/// W64 = WoW64 64-Key
/// W32 = WoW64 32-Key
/// ```
#[bitfield::bitfield(32)]
#[derive(Debug)]
struct AccessRegistry {
    object: Registry,
    standard: Standard
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Registry {
    GetValue,
    SetValue,
    CreateSubKey,
    ListSubKeys,
    Notify,
    CreateLink,
    // Bits 6 and 7 are reserved.
    WoW64Key64 = 8,
    WoW64Key32
    // Bits 10 - 15 are reserved.
}

/// [Microsoft Docs: Thread Access Rights Constants](https://docs.microsoft.com/en-us/windows/desktop/ProcThread/thread-security-and-access-rights)
///
/// Layout:
///
/// ```
///  15      11      7       3     0
/// ╔═══════╪═╤═╤═╤═╪═╤═╤═╤═╪═╤═╤═╤═╗
/// ║       │G│S│D│I│S│G│S│S│G│A│S│T║
/// ║       │L│L│I│ │T│I│I│C│C│ │R│ ║ Thread
/// ║0 0 0 0│I│I│ │ │ │ │ │ │ │ │ │ ║
/// ╚═══════╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╧═╝
/// T   = Terminate
/// SR  = Suspend + Resume
/// A   = Alert
/// GC  = Get Context
/// SC  = Set Context
/// SI  = Set Information
/// GI  = Get Information
/// ST  = Set Token
/// I   = Impersonate
/// DI  = Direct Impersonation
/// SLT = Set Limited Information
/// GLT = Get Limited Information
/// ```
#[bitfield::bitfield(32)]
#[derive(Debug)]
struct AccessThread {
    object: Thread,
    standard: Standard
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Thread {
    Terminate,
    SuspendResume,
    Alert,
    GetContext,
    SetContext,
    SetInformation,
    GetInformation,
    SetToken,
    Impersonate,
    DirectImpersonation,
    SetLimitedInformation,
    GetLimitedInformation
    // Bits 12 - 15 are reserved.
}

fn main() {
    let directory = AccessDirectory::new()
        + Directory::List        // Same as: `.set_object(Directory::List, true)`
        + Directory::Traverse    // Same as: `.set_object(Directory::Traverse, true)`
        + Standard::Delete       // Same as: `.set_standard(Standard::Delete, true)`
        + Standard::Synchronize; // Same as: `.set_standard(Standard::Synchronize, true)`

    let file = AccessFile::new()
        + File::Write            // Same as: `.set_object(File::Write, true)`
        + File::WriteAttributes  // Same as: `.set_object(File::WriteAttributes, true)`
        + Standard::Delete       // Same as: `.set_standard(Standard::Delete, true)`
        + Standard::Synchronize; // Same as: `.set_standard(Standard::Synchronize, true)`

    for flag in Standard::iter() {
        assert_eq!(
            directory.standard(*flag),
            file.standard(*flag)
        );
    }

    println!("{:#?}", &directory);
    println!("{:#?}", &file);
}