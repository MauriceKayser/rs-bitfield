#![allow(non_camel_case_types)]

extern crate alloc;

/// [struct GNU::GIOChannel](https://github.com/GNOME/glib/blob/master/glib/giochannel.h)
///
/// Based on a comment in [#RFC-1449 Add language support for bitfields](https://github.com/rust-lang/rfcs/pull/1449#issuecomment-318482265).
///
/// Layout (only the flags):
///
/// ```
///  7       3     0
/// ╔═══╤═╤═╪═╤═╤═╤═╗
/// ║   │I│I│I│C│D│U║
/// ║   │S│W│R│O│E│B║ Flags
/// ║0 0│ │ │ │U│ │ ║
/// ╚═══╧═╧═╧═╧═╧═╧═╝
/// UB  = Use Buffer
/// DE  = Do Encode
/// COU = Close On Unref
/// IR  = Is Readable
/// IW  = Is Writable
/// IS  = Is Seekable
/// ```
#[bitfield::bitfield(8)]
#[derive(Display)]
struct Flags(Flag);

#[derive(Copy, Clone, Debug, bitfield::Flags)]
#[repr(u8)]
enum Flag {
    /// The encoding uses the buffers.
    UseBuffer,
    /// The encoding uses the `GIConv` converters.
    DoEncode,
    /// Close the channel on final unref.
    CloseOnUnref,
    /// Cached `GIOFlag`.
    IsReadable,
    /// Cached `GIOFlag`.
    IsWritable,
    /// Cached `GIOFlag`.
    IsSeekable
    // Bits 6 - 7 are reserved.
}

/// See https://github.com/GNOME/glib/blob/master/glib/giochannel.h
#[repr(C)]
struct GIOChannel {
    ref_count: gint,
    funcs: *const u8,

    encoding: *const gchar,
    read_cd: GIConv,
    write_cd: GIConv,
    /// String which indicates the end of a line of text.
    line_term: *const gchar,
    /// So we can have null in the line term.
    line_term_len: guint,

    buf_size: gsize,
    /// Raw data from the channel.
    read_buf: *const GString,
    /// Channel data converted to UTF-8.
    encoded_read_buf: *const GString,
    /// Data ready to be written to the file.
    write_buf: *const GString,
    /// UTF-8 partial characters, null terminated.
    partial_write_buf: [gchar; 6],

    // Group the flags together, immediately after `partial_write_buf`, to save memory.
    flags: Flags,

    reserved1: gpointer,
    reserved2: gpointer
}

/// See https://github.com/GNOME/glib/blob/master/glib/gtypes.h
type gchar = i8;
type gint = i32;
type gpointer = *const u8;
type gsize = usize;
type guint = u32;

/// See https://github.com/GNOME/glib/blob/master/glib/gconvert.h
type GIConv = *const u8;

/// See https://github.com/GNOME/glib/blob/master/glib/gstring.h
#[repr(C)]
struct GString {
    str: *const gchar,
    len: gsize,
    allocated_len: gsize
}

fn main() {
    let mut channel = GIOChannel {
        ref_count: 0, funcs: 0 as _, encoding: 0 as _, read_cd: 0 as _, write_cd: 0 as _,
        line_term: 0 as _, line_term_len: 0, buf_size: 0, read_buf: 0 as _,
        encoded_read_buf: 0 as _, write_buf: 0 as _, partial_write_buf: [0; 6],
        reserved1: 0 as _, reserved2: 0 as _,

        flags: Flags::new()
    };

    // Check and update flags.
    channel.flags = channel.flags
        .set(Flag::DoEncode, true)
        .set(Flag::IsWritable, true);

    if !channel.flags.has(Flag::CloseOnUnref) {
        channel.flags = channel.flags.set(Flag::CloseOnUnref, true);
    }

    assert_eq!(&format!("{}", &channel.flags), "DoEncode | CloseOnUnref | IsWritable");

    println!("Flags: {}", &channel.flags);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size() {
        #[cfg(target_pointer_width = "32")]
        assert_eq!(
            core::mem::size_of::<GIOChannel>(),
            4 +     // ref_count: gint,
            4 +     // funcs: *const u8,
            4 +     // encoding: *const gchar,
            4 +     // read_cd: GIConv,
            4 +     // write_cd: GIConv,
            4 +     // line_term: *const gchar,
            4 +     // line_term_len: guint,
            4 +     // buf_size: gsize,
            4 +     // read_buf: *const GString,
            4 +     // encoded_read_buf: *const GString,
            4 +     // write_buf: *const GString,
            6 +     // partial_write_buf: [gchar; 6],
            1 +     // flags: Flags,
            (1) +   // -- alignment --
            4 +     // reserved1: gpointer,
            4       // reserved2: gpointer
        );

        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            core::mem::size_of::<GIOChannel>(),
            4 +     // ref_count: gint,
            (4) +   // -- alignment --
            8 +     // funcs: *const u8,
            8 +     // encoding: *const gchar,
            8 +     // read_cd: GIConv,
            8 +     // write_cd: GIConv,
            8 +     // line_term: *const gchar,
            4 +     // line_term_len: guint,
            (4) +   // -- alignment --
            8 +     // buf_size: gsize,
            8 +     // read_buf: *const GString,
            8 +     // encoded_read_buf: *const GString,
            8 +     // write_buf: *const GString,
            6 +     // partial_write_buf: [gchar; 6],
            1 +     // flags: Flags,
            (1) +   // -- alignment --
            8 +     // reserved1: gpointer,
            8       // reserved2: gpointer
        );
    }
}