extern crate alloc;

/// [Microsoft Docs: MessageBox function](https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-messagebox)
///
/// The contents and behavior of the dialog box.
///
/// Layout:
///
/// ```
///  31      27      23      19      15      11      7       3     0
/// ╔═══════╧═══════╧═══╤═╤═╪═╤═╤═╤═╪═╤═╤═══╪═══════╪═══════╪═══════╗
/// ║                   │S│R│R│T│D│F│ │H│Mod│DefBtn │Icon   │Button ║
/// ║                   │N│T│ │M│D│ │ │ │   │       │       │       ║ Styles
/// ║0 0 0 0 0 0 0 0 0 0│ │L│ │ │O│ │0│ │   │       │       │       ║
/// ╚═══════════════════╧═╧═╧═╧═╧═╧═╧═╧═╧═══╧═══════╧═══════╧═══════╝
///          Button
///          Icon
/// DefBtn = Default Button
/// Mod    = Modality
/// H      = Help
/// F      = Foreground
/// DDO    = Default Desktop Only
/// TM     = Top Most
/// R      = Right
/// RTL    = Right To Left Reading
/// SN     = Service Notification
/// ```
#[bitfield::bitfield(32)]
#[derive(Debug)]
struct Styles {
    #[field(size = 4)] button: Button,
    #[field(size = 4)] icon: Icon,
    #[field(size = 4)] default_button: DefaultButton,
    #[field(size = 2)] modality: Modality,
    style: Style
}

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Style {
    Help = 14,
    // Bit 15 is reserved.
    Foreground = 16,
    DefaultDesktopOnly,
    TopMost,
    Right,
    RightToLeftReading,
    ServiceNotification
    // Bits 22 - 31 are reserved.
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum Button {
    Ok,
    OkCancel,
    AbortRetryIgnore,
    YesNoCancel,
    YesNo,
    RetryCancel,
    CancelTryContinue
    // Variants 7 - 15 are reserved.
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum DefaultButton {
    One,
    Two,
    Three,
    Four
    // Variants 4 - 15 are reserved.
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum Icon {
    None,
    Stop,
    Question,
    Exclamation,
    Information
    // Variants 5 - 15 are reserved.
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum Modality {
    Application,
    System,
    Task
    // Variant 3 is reserved.
}

fn main() {
    let styles = Styles::new()
        + Button::CancelTryContinue // Same as: `.set_button(Button::CancelTryContinue)`
        + Icon::Exclamation         // Same as: `.set_icon(Icon::Exclamation)`
        + DefaultButton::Two        // Same as: `.set_default_button(DefaultButton::Two)`
        + Modality::Task            // Same as: `.set_modality(Modality::Task)`
        + Style::Foreground         // Same as: `.set_style(Style::Foreground, true)`
        + Style::TopMost;           // Same as: `.set_style(Style::TopMost, true)`

    assert_eq!(
        styles.0,

        (6 << 0) +   // button
        (3 << 4) +   // icon
        (1 << 8) +   // default_button
        (2 << 12) +  // modality
        (1 << 16) +  // Style::Foreground
        (1 << 18)    // Style::TopMost
    );

    assert_eq!(&format!("{:#?}", &styles),
"Styles {
    button: CancelTryContinue,
    icon: Exclamation,
    default_button: Two,
    modality: Task,
    style: Style {
        Help: false,
        Foreground: true,
        DefaultDesktopOnly: false,
        TopMost: true,
        Right: false,
        RightToLeftReading: false,
        ServiceNotification: false,
    },
}"
    );

    println!("{:#?}", &styles);
}