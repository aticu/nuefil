//! This protocol is used to handle input and output of
//! text-based information intended for the system user during the operation of code in the boot
//! services environment. Also included here are the definitions of three console devices: one for input
//! and one each for normal output and errors.

use core::fmt;

use crate::{
    status::{Error, Status, Warning},
    system::SystemTable,
    Event,
};

/// Keystroke information for the key that was pressed.
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct TextInputKey {
    /// If there is a pending keystroke, then ScanCode is the EFI scan code defined in
    /// Table 104.
    pub ScanCode: u16,
    /// The UnicodeChar is the actual printable
    /// character or is zero if the key does not represent a printable
    /// character (control key, function key, etc.).
    pub UnicodeChar: u16,
}

/// This protocol is used to obtain input from the ConsoleIn device. The EFI specification requires that
/// the EFI_SIMPLE_TEXT_INPUT_PROTOCOL supports the same languages as the corresponding
/// EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL.
#[repr(C)]
pub struct TextInput {
    /// Reset the ConsoleIn device.
    pub Reset: extern "win64" fn(&TextInput, bool) -> Status,
    /// Returns the next input character.
    pub ReadKeyStroke: extern "win64" fn(&TextInput, &mut TextInputKey) -> Status,
    /// Event to use with EFI_BOOT_SERVICES.WaitForEvent() to wait for a key to be available.
    pub WaitForKey: Event,
}

impl TextInput {
    /// Reset the ConsoleOut device.
    pub fn reset(&self, extended_verification: bool) -> Result<(), Error> {
        (self.Reset)(self, extended_verification)?;

        Ok(())
    }

    /// Returns the next input character, if it exists.
    pub fn try_read_key_stroke(&self) -> Result<TextInputKey, Error> {
        let mut key = TextInputKey::default();

        (self.ReadKeyStroke)(self, &mut key)?;

        Ok(key)
    }

    /// Returns the next input character after waiting for it.
    pub fn read_key_stroke(
        &self,
        system_table: &'static SystemTable,
    ) -> Result<TextInputKey, Error> {
        system_table.BootServices.wait_for_event(&self.WaitForKey)?;

        self.try_read_key_stroke()
    }
}

/// The following data values in the SIMPLE_TEXT_OUTPUT_MODE interface are read-only and are
/// changed by using the appropriate interface functions.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TextOutputMode {
    /// The number of modes supported by QueryMode() and SetMode().
    pub MaxMode: i32,
    /// The text mode of the output device(s).
    pub Mode: i32,
    /// The current character output attribute.
    pub Attribute: i32,
    /// The cursor’s column.
    pub CursorColumn: i32,
    /// The cursor’s row.
    pub CursorRow: i32,
    /// The cursor is currently visible or not.
    pub CursorVisible: bool,
}

/// This protocol is used to control text-based output devices.
#[repr(C)]
pub struct TextOutput {
    /// Reset the ConsoleOut device.
    pub Reset: extern "win64" fn(&TextOutput, bool) -> Status,
    /// Displays the string on the device at the current cursor location.
    pub OutputString: extern "win64" fn(&TextOutput, *const u16) -> Status,
    /// Tests to see if the ConsoleOut device supports this string.
    pub TestString: extern "win64" fn(&TextOutput, *const u16) -> Status,
    /// Queries information concerning the output device’s supported text mode.
    pub QueryMode: extern "win64" fn(&TextOutput, usize, &mut usize, &mut usize) -> Status,
    /// Sets the current mode of the output device.
    pub SetMode: extern "win64" fn(&TextOutput, usize) -> Status,
    /// Sets the foreground and background color of the text that is output.
    pub SetAttribute: extern "win64" fn(&TextOutput, usize) -> Status,
    /// Clears the screen with the currently set background color.
    pub ClearScreen: extern "win64" fn(&TextOutput) -> Status,
    /// Sets the current cursor position.
    pub SetCursorPosition: extern "win64" fn(&TextOutput, usize, usize) -> Status,
    /// Turns the visibility of the cursor on/off.
    pub EnableCursor: extern "win64" fn(&TextOutput, bool) -> Status,
    /// Reference to SIMPLE_TEXT_OUTPUT_MODE data.
    pub Mode: &'static TextOutputMode,
}

impl TextOutput {
    /// Reset the ConsoleOut device.
    pub fn reset(&self, extended_verification: bool) -> Result<(), Error> {
        (self.Reset)(self, extended_verification)?;

        Ok(())
    }

    /// Displays the string on the device at the current cursor location.
    pub fn output_string(&self, string: &str) -> Result<Warning, Error> {
        with_utf16_str(string, |utf16| (self.OutputString)(self, utf16))
    }

    /// Tests to see if the ConsoleOut device supports this string.
    pub fn test_string(&self, string: &str) -> Result<(), Error> {
        with_utf16_str(string, |utf16| (self.TestString)(self, utf16))?;

        Ok(())
    }

    /// Queries information concerning the output device’s supported text mode.
    ///
    /// Returns the number of columns and rows, if successful.
    pub fn query_mode(&self, mode_number: usize) -> Result<(usize, usize), Error> {
        let mut columns: usize = 0;
        let mut rows: usize = 0;

        (self.QueryMode)(self, mode_number, &mut columns, &mut rows)?;

        Ok((columns, rows))
    }

    /// Sets the current mode of the output device.
    pub fn set_mode(&self, mode: usize) -> Result<(), Error> {
        (self.SetMode)(self, mode)?;

        Ok(())
    }

    /// Sets the foreground and background color of the text that is output.
    pub fn set_attribute(&self, attribute: Color) -> Result<(), Error> {
        (self.SetAttribute)(self, attribute.0)?;

        Ok(())
    }

    /// Clears the screen with the currently set background color.
    pub fn clear_screen(&self) -> Result<(), Error> {
        (self.ClearScreen)(self)?;

        Ok(())
    }

    /// Sets the current cursor position.
    pub fn set_cursor_position(&self, column: usize, row: usize) -> Result<(), Error> {
        (self.SetCursorPosition)(self, column, row)?;

        Ok(())
    }

    /// Turns the visibility of the cursor on/off.
    pub fn enable_cursor(&self, enable: bool) -> Result<(), Error> {
        (self.EnableCursor)(self, enable)?;

        Ok(())
    }
}

impl<'a> fmt::Write for &'a TextOutput {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.output_string(s).map_err(|_| fmt::Error).map(|_| ())
    }
}

/// Executes the given function with the UTF16-encoded string.
///
/// `function` will get a UTF16-encoded null-terminated string as its argument when its called.
///
/// `function` may be called multiple times. This is because the string is converted
/// to UTF16 in a fixed size buffer to avoid allocations. As a consequence, any string
/// longer than the buffer needs the function to be called multiple times.
fn with_utf16_str<FunctionType>(string: &str, function: FunctionType) -> Result<Warning, Error>
where
    FunctionType: Fn(*const u16) -> Status,
{
    const BUFFER_SIZE: usize = 256;

    let mut buffer = [0u16; BUFFER_SIZE];

    let mut current_index = 0;

    let warning = Warning::Success;

    // Flushes the buffer
    let flush_buffer = |ref mut buffer: &mut [u16; BUFFER_SIZE],
                        ref mut current_index,
                        ref mut warning|
     -> Result<(), Error> {
        buffer[*current_index] = 0;
        *current_index = 0;

        if *warning == Warning::Success {
            *warning = function(buffer.as_ptr())?;
        } else {
            function(buffer.as_ptr())?;
        }

        Ok(())
    };

    for character in string.chars() {
        // If there is not enough space in the buffer, flush it
        if current_index + character.len_utf16() + 1 >= BUFFER_SIZE {
            flush_buffer(&mut buffer, current_index, warning)?;
        }

        character.encode_utf16(&mut buffer[current_index..]);
        current_index += character.len_utf16();
    }

    flush_buffer(&mut buffer, current_index, warning)?;

    Ok(warning)
}

/// Represents color information for text.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Color(usize);

impl Color {
    /// Creates new color information from the foreground and background color.
    pub const fn new(foreground: ForegroundColor, background: BackgroundColor) -> Color {
        Color(foreground as usize | background as usize)
    }
}

/// Represents a foreground color for text.
#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum ForegroundColor {
    /// Represents the foreground color black.
    Black = 0x00,
    /// Represents the foreground color blue.
    Blue = 0x01,
    /// Represents the foreground color green.
    Green = 0x02,
    /// Represents the foreground color cyan.
    Cyan = 0x03,
    /// Represents the foreground color red.
    Red = 0x04,
    /// Represents the foreground color magenta.
    Magenta = 0x05,
    /// Represents the foreground color brown.
    Brown = 0x06,
    /// Represents the foreground color light gray.
    LightGray = 0x07,
    /// Represents the foreground color dark gray.
    DarkGray = 0x08,
    /// Represents the foreground color light blue.
    LightBlue = 0x09,
    /// Represents the foreground color light green.
    LightGreen = 0x0a,
    /// Represents the foreground color light cyan.
    LightCyan = 0x0b,
    /// Represents the foreground color light red.
    LightRed = 0x0c,
    /// Represents the foreground color light magenta.
    LightMagenta = 0x0d,
    /// Represents the foreground color yellow.
    Yellow = 0x0e,
    /// Represents the foreground color white.
    White = 0x0f,
}

/// Represents a background color for text.
#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum BackgroundColor {
    /// Represents the background color black.
    Black = 0x00,
    /// Represents the background color blue.
    Blue = 0x10,
    /// Represents the background color green.
    Green = 0x20,
    /// Represents the background color cyan.
    Cyan = 0x30,
    /// Represents the background color red.
    Red = 0x40,
    /// Represents the background color magenta.
    Magenta = 0x50,
    /// Represents the background color brown.
    Brown = 0x60,
    /// Represents the background color light gray.
    LightGray = 0x70,
}
