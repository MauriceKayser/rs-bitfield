extern crate alloc;

/// [Writing an OS in Rust - VGA Text Mode](https://os.phil-opp.com/vga-text-mode/)
///
/// The contents of a single screen character.
///
/// Layout:
///
/// ```
///  7       3     0
/// ╔═╤═════╪═╤═════╗
/// ║B│BGCol│B│FGCol║
/// ║ │     │F│     ║ Styles
/// ║ │     │ │     ║
/// ╚═╧═════╧═╧═════╝
/// FGCol = Foreground Color
/// BF    = Bright Foreground
/// BGCol = Background Color
/// B     = Blink
/// ```
#[bitfield::bitfield(8)]
#[derive(Debug)]
struct Styles {
    #[field(size = 3)]
    foreground: Color,
    foreground_bright: bool,
    #[field(size = 3)]
    background: Color,
    blink: bool
}

#[derive(Clone, Copy, Debug, bitfield::Field)]
#[repr(u8)]
enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    Gray
}

#[derive(Debug)]
struct ScreenChar {
    ascii_character: u8,
    styles: Styles
}

fn main() {
    let mut char = ScreenChar { ascii_character: b'>', styles: Styles::new() };

    // Check and update flags.
    char.styles = char.styles
        .set_foreground(Color::Green)
        .set_background(Color::Black)
        .set_blink(true);

    if !char.styles.foreground_bright() {
        char.styles = char.styles.invert_foreground_bright();
    }

    assert_eq!(&format!("{:#?}", &char),
"ScreenChar {
    ascii_character: 62,
    styles: Styles {
        foreground: Green,
        foreground_bright: true,
        background: Black,
        blink: true,
    },
}"
    );

    println!("{:#?}", &char);
}