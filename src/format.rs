use std::cmp::PartialEq;
use std::fmt;
use std::fmt::Write;

/// A fast JSON formatter / pretty printer.
/// This is a fast JSON formatter (x2 compared to pretty printing with [Serde JSON](https://github.com/serde-rs/json)).
/// This formatter parses and formats JSON input byte by byte and do not require pre UTF-8 validation.
/// UTF-8 validation is done in-place, on the fly, while parsing strings. This implementation try to not allocate
/// anything. It does not try to normalise, remove unnecessary escaping, it just formats the actual input
/// with spaces, newlines and (optionally) color.
pub struct Formatter<'input> {
    /// The JSON input bytes to prettify.
    input: &'input [u8],
    /// Cursor position in byte offset.
    pos: BytePos,
    /// Current indentation level (this is maxed by `MAX_INDENT_LEVEL`)
    level: usize,
    /// Use color with ANSI escape code when prettifying.
    color: Color,
}

/// The maximum indentation level supported before errors.
const MAX_INDENT_LEVEL: usize = 100;

/// A byte position in a bytes stream.
#[derive(Debug, Copy, Clone)]
pub struct BytePos(usize);

/// Potential errors raised during formatting.
#[derive(Debug)]
pub enum FormatError {
    /// Unexpected end of file.
    Eof,
    /// Invalid byte at this position.
    InvalidByte(u8, BytePos),
    /// The next bytes are not a valid UTF-8 sequence.
    InvalidUtf8([u8; 4], usize, BytePos),
    /// Invalid escaped byte at this position.
    InvalidEscape(u8, BytePos),
    /// The maximum indent level has been reached.
    MaxIndentLevel(usize, BytePos),
    Fmt(fmt::Error),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Eof => write!(f, "unexpected end of file"),
            FormatError::InvalidByte(byte, pos) => {
                write!(f, "invalid byte <{byte:02x?}> at offset {}", pos.0)
            }
            FormatError::InvalidUtf8(bytes, len, pos) => {
                let hex = bytes
                    .iter()
                    .take(*len)
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "invalid {} UTF-8 bytes <{hex}> at offset {}", len, pos.0)
            }
            FormatError::InvalidEscape(byte, pos) => {
                write!(f, "invalid escaped byte <{byte:02x?}> at offset {}", pos.0)
            }
            FormatError::MaxIndentLevel(level, pos) => {
                write!(f, "maximum indent level {} at offset {}", level, pos.0)
            }
            FormatError::Fmt(error) => write!(f, "error writing {error}"),
        }
    }
}

impl From<fmt::Error> for FormatError {
    fn from(e: fmt::Error) -> Self {
        FormatError::Fmt(e)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
    NoColor,
    AnsiCode,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum StringMode {
    Key,
    Value,
}

type FormatResult<T> = Result<T, FormatError>;

impl<'input> Formatter<'input> {
    pub fn new(input: &'input [u8], color: Color) -> Self {
        Formatter {
            input,
            pos: BytePos(0),
            level: 0,
            color,
        }
    }

    #[inline]
    fn next_byte(&mut self) -> Option<u8> {
        let b = self.peek_byte()?;
        self.pos.0 += 1;
        Some(b)
    }

    #[inline]
    fn peek_byte(&mut self) -> Option<u8> {
        self.input.get(self.pos.0).copied()
    }

    #[inline]
    fn expect_byte(&mut self, expected: u8) -> FormatResult<()> {
        match self.next_byte() {
            Some(b) if b == expected => Ok(()),
            Some(b) => Err(FormatError::InvalidByte(b, BytePos(self.pos.0 - 1))),
            None => Err(FormatError::Eof),
        }
    }

    fn inc_level(&mut self) -> FormatResult<()> {
        if self.level >= MAX_INDENT_LEVEL {
            return Err(FormatError::MaxIndentLevel(self.level, self.pos));
        }
        self.level += 1;
        Ok(())
    }

    fn dec_level(&mut self) {
        self.level -= 1;
    }


    /// Formats and colorize the JSON input bytes.
    pub fn format(&mut self, out: &mut impl Write) -> FormatResult<()> {
        self.skip_start_bom();

        self.skip_whitespace();
        self.parse_value(out)?;
        self.skip_whitespace();

        // Have we completely consumed our payload?
        if let Some(b) = self.peek_byte() {
            Err(FormatError::InvalidByte(b, self.pos))
        } else {
            Ok(())
        }
    }

    /// Skips BOM (Byte Order Mark) at the start of the read buffer.
    fn skip_start_bom(&mut self) {
        debug_assert!(self.pos.0 == 0);
        if self.input.len() < 3 {
            return;
        }
        if self.input[0] == 0xEF && self.input[1] == 0xBB && self.input[2] == 0xBF {
            self.pos.0 = 3;
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos.0 += 1;
        }
    }

    /// Value
    fn parse_value(&mut self, out: &mut impl Write) -> FormatResult<()> {
        // From <https://datatracker.ietf.org/doc/html/rfc7159#section-3>:
        //
        // value = false / null / true / object / array / number / string
        // false = %x66.61.6c.73.65   ; false
        // null  = %x6e.75.6c.6c      ; null
        // true  = %x74.72.75.65      ; true
        match self.peek_byte() {
            Some(b'"') => self.parse_string(out, StringMode::Value),
            Some(b'-' | b'0'..=b'9') => self.parse_number(out),
            Some(b'{') => self.parse_object(out),
            Some(b'[') => self.parse_array(out),
            Some(b't') => self.parse_true(out),
            Some(b'f') => self.parse_false(out),
            Some(b'n') => self.parse_null(out),
            Some(b) => Err(FormatError::InvalidByte(b, self.pos)),
            None => Err(FormatError::Eof),
        }
    }

    /// Object
    fn parse_object(&mut self, out: &mut impl Write) -> FormatResult<()> {
        // From <https://datatracker.ietf.org/doc/html/rfc7159#section-4>:
        // object = begin-object [ member *( value-separator member ) ]
        // end-object
        // member = string name-separator value
        self.expect_byte(b'{')?;

        // For empty objects, we keep a short compact form:
        self.skip_whitespace();
        if self.peek_byte() == Some(b'}') {
            self.next_byte();
            self.write_empty_obj(out)?;
            return Ok(());
        }

        // Now, we have a non-empty object.
        self.write_begin_obj(out)?;
        self.inc_level()?;

        let mut first = true;
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.next_byte();
                self.dec_level();
                self.write_ln(out)?;
                self.write_indent(out)?;
                self.write_end_obj(out)?;
                return Ok(());
            }

            if first {
                first = false;
            } else {
                self.expect_byte(b',')?;
                self.skip_whitespace();
                self.write_value_sep(out)?;
            }

            // Parse key
            self.write_indent(out)?;
            self.parse_string(out, StringMode::Key)?;

            // Parse colon
            self.skip_whitespace();
            self.expect_byte(b':')?;
            self.write_name_sep(out)?;

            // Parse value
            self.skip_whitespace();
            self.parse_value(out)?;
        }
    }

    /// Array
    fn parse_array(&mut self, out: &mut impl Write) -> FormatResult<()> {
        // From <https://datatracker.ietf.org/doc/html/rfc7159#section-4>:
        // array = begin-array [ value *( value-separator value ) ] end-array
        self.expect_byte(b'[')?;

        // For empty arrays, we keep a short compact form:
        self.skip_whitespace();
        if self.peek_byte() == Some(b']') {
            self.next_byte();
            self.write_empty_arr(out)?;
            return Ok(());
        }

        // Now, we have a non-empty array.
        self.write_begin_arr(out)?;
        self.inc_level()?;

        let mut first = true;
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b']') {
                self.next_byte();
                self.dec_level();
                self.write_ln(out)?;
                self.write_indent(out)?;
                self.write_end_arr(out)?;
                return Ok(());
            }

            if first {
                first = false;
            } else {
                self.expect_byte(b',')?;
                self.skip_whitespace();
                self.write_value_sep(out)?;
            }

            self.write_indent(out)?;
            self.parse_value(out)?;
        }
    }

    fn slice_str_unchecked(&self, start: BytePos, end: BytePos) -> &str {
        debug_assert!(start.0 <= end.0 && end.0 <= self.input.len());
        let bytes = &self.input[start.0..end.0];
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    /// String (preserves escapes)
    fn parse_string(&mut self, out: &mut impl Write, mode: StringMode) -> FormatResult<()> {
        // From <https://datatracker.ietf.org/doc/html/rfc7159#section-8>

        let start = self.pos;
        self.expect_byte(b'"')?;

        while let Some(b) = self.peek_byte() {
            match b {
                b'"' => {
                    self.next_byte();

                    // Flush plain segment before exit.
                    let string = self.slice_str_unchecked(start, self.pos);
                    match mode {
                        StringMode::Key => self.write_key(string, out)?,
                        StringMode::Value => self.write_value(string, out)?,
                    };
                    return Ok(());
                }
                // Escaping
                b'\\' => {
                    self.next_byte();
                    match self.next_byte() {
                        Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => {}
                        Some(b'u') => {
                            for _ in 0..4 {
                                let hex = self.next_byte().ok_or(FormatError::Eof)?;
                                if !(hex as char).is_ascii_hexdigit() {
                                    return Err(FormatError::InvalidByte(
                                        hex,
                                        BytePos(self.pos.0 - 1),
                                    ));
                                }
                            }
                        }
                        Some(b) => return Err(FormatError::InvalidEscape(b, self.pos)),
                        None => return Err(FormatError::Eof),
                    }
                }
                0x00..=0x1F => return Err(FormatError::InvalidByte(b, self.pos)),
                _ => {
                    // Decode valid UTF-8 char
                    self.next_utf8_char()?;
                }
            }
        }
        Err(FormatError::Eof)
    }

    /// Literals
    fn parse_true(&mut self, out: &mut impl Write) -> FormatResult<()> {
        for &b in b"true" {
            self.expect_byte(b)?;
        }
        self.write_true(out)?;
        Ok(())
    }

    fn parse_false(&mut self, out: &mut impl Write) -> FormatResult<()> {
        for &b in b"false" {
            self.expect_byte(b)?;
        }
        self.write_false(out)?;
        Ok(())
    }

    fn parse_null(&mut self, out: &mut impl Write) -> FormatResult<()> {
        for &b in b"null" {
            self.expect_byte(b)?;
        }
        self.write_null(out)?;
        Ok(())
    }

    /// Parse a JSON number.
    fn parse_number(&mut self, out: &mut impl Write) -> FormatResult<()> {
        // From the spec <https://datatracker.ietf.org/doc/html/rfc7159#section-6>:
        //
        // number = [ minus ] int [ frac ] [ exp ]
        // decimal-point = %x2E       ; .
        // digit1-9 = %x31-39         ; 1-9
        // e = %x65 / %x45            ; e E
        // exp = e [ minus / plus ] 1*DIGIT
        // frac = decimal-point 1*DIGIT
        // int = zero / ( digit1-9 *DIGIT )
        // minus = %x2D               ; -
        // plus = %x2B                ; +
        // zero = %x30                ; 0

        let start = self.pos;

        if self.peek_byte() == Some(b'-') {
            self.next_byte();
        }

        self.parse_integer()?;
        self.parse_fraction()?;
        self.parse_exponent()?;

        // Finally, write numbers
        let digits = self.slice_str_unchecked(start, self.pos);
        self.write_number(digits, out)?;

        Ok(())
    }

    fn parse_integer(&mut self) -> FormatResult<()> {
        match self.peek_byte() {
            Some(b'0') => {
                self.next_byte();
                Ok(())
            }
            Some(b'1'..=b'9') => {
                self.next_byte();
                // 0 or more digits
                while let Some(b'0'..=b'9') = self.peek_byte() {
                    self.next_byte();
                }
                Ok(())
            }
            Some(b) => Err(FormatError::InvalidByte(b, self.pos)),
            None => Err(FormatError::Eof),
        }
    }

    fn parse_fraction(&mut self) -> FormatResult<()> {
        if self.peek_byte() == Some(b'.') {
            self.next_byte();
            // 1 or more digits
            match self.peek_byte() {
                Some(b'0'..=b'9') => {
                    self.next_byte();
                    while let Some(b'0'..=b'9') = self.peek_byte() {
                        self.next_byte();
                    }
                    Ok(())
                }
                Some(b) => Err(FormatError::InvalidByte(b, self.pos)),
                None => Err(FormatError::Eof),
            }?
        }
        Ok(())
    }

    fn parse_exponent(&mut self) -> FormatResult<()> {
        match self.peek_byte() {
            Some(b'e' | b'E') => {
                self.next_byte();
                if let Some(b'+' | b'-') = self.peek_byte() {
                    self.next_byte();
                }
                match self.peek_byte() {
                    Some(b'0'..=b'9') => {
                        self.next_byte();
                        while let Some(b'0'..=b'9') = self.peek_byte() {
                            self.next_byte();
                        }
                        Ok(())
                    }
                    Some(b) => Err(FormatError::InvalidByte(b, self.pos)),
                    None => Err(FormatError::Eof),
                }
            }
            _ => Ok(()),
        }
    }

    // -------- UTF-8 decoder --------
    fn next_utf8_char(&mut self) -> FormatResult<()> {
        #[inline(always)]
        fn cont(b: u8) -> bool {
            (b & 0xC0) == 0x80
        }

        let start_pos = self.pos;

        let b1 = self.next_byte().ok_or(FormatError::Eof)?;
        if b1 < 0x80 {
            return Ok(());
        }

        let b2 = self.next_byte().ok_or(FormatError::Eof)?;
        if b1 < 0xE0 {
            return if (0xC2..=0xDF).contains(&b1) && cont(b2) {
                Ok(())
            } else {
                Err(FormatError::InvalidUtf8([b1, b2, 0, 0], 2, start_pos))
            };
        }

        let b3 = self.next_byte().ok_or(FormatError::Eof)?;
        if b1 < 0xF0 {
            return if match b1 {
                0xE0 => (0xA0..=0xBF).contains(&b2) && cont(b3),
                0xED => (0x80..=0x9F).contains(&b2) && cont(b3), // no surrogates
                0xE1..=0xEC | 0xEE..=0xEF => cont(b2) && cont(b3),
                _ => false,
            } {
                Ok(())
            } else {
                Err(FormatError::InvalidUtf8([b1, b2, b3, 0], 3, self.pos))
            };
        }

        let b4 = self.next_byte().ok_or(FormatError::Eof)?;
        if match b1 {
            0xF0 => (0x90..=0xBF).contains(&b2) && cont(b3) && cont(b4),
            0xF4 => (0x80..=0x8F).contains(&b2) && cont(b3) && cont(b4),
            0xF1..=0xF3 => cont(b2) && cont(b3) && cont(b4),
            _ => false,
        } {
            Ok(())
        } else {
            Err(FormatError::InvalidUtf8([b1, b2, b3, b4], 4, self.pos))
        }
    }
}

const SPACES: &str = "                                                                 ";

/// Methods to print on a [Write], with color, or not.
impl<'input> Formatter<'input> {
    fn write_indent(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        let n = self.level * 2;
        let full_chunks = n / SPACES.len();
        let remainder = n % SPACES.len();
        for _ in 0..full_chunks {
            out.write_str(SPACES)?;
        }
        out.write_str(&SPACES[..remainder])?;
        Ok(())
    }

    #[inline]
    fn write_ln(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        out.write_char('\n')
    }

    #[inline]
    fn write_empty_obj(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[1;39m{}\x1b[0m")
        } else {
            out.write_str("{}")
        }
    }

    #[inline]
    fn write_begin_obj(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[1;39m{\x1b[0m\n")
        } else {
            out.write_str("{\n")
        }
    }

    #[inline]
    fn write_end_obj(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[1;39m}\x1b[0m")
        } else {
            out.write_char('}')
        }
    }

    #[inline]
    fn write_value_sep(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[1;39m,\x1b[0m\n")
        } else {
            out.write_str(",\n")
        }
    }

    #[inline]
    fn write_name_sep(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[1;39m:\x1b[0m ")
        } else {
            out.write_str(": ")
        }
    }

    #[inline]
    fn write_empty_arr(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[1;39m[]\x1b[0m")
        } else {
            out.write_str("[]")
        }
    }

    #[inline]
    fn write_begin_arr(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[1;39m[\x1b[0m\n")
        } else {
            out.write_str("[\n")
        }
    }

    #[inline]
    fn write_end_arr(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[1;39m]\x1b[0m")
        } else {
            out.write_str("]")
        }
    }

    #[inline]
    fn write_key(&self, s: &str, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[1;34m")?;
            out.write_str(s)?;
            out.write_str("\x1b[0m")
        } else {
            out.write_str(s)
        }
    }

    #[inline]
    fn write_value(&self, s: &str, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[0;32m")?;
            out.write_str(s)?;
            out.write_str("\x1b[0m")
        } else {
            out.write_str(s)
        }
    }

    #[inline]
    fn write_true(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[0;33mtrue\x1b[0m")
        } else {
            out.write_str("true")
        }
    }

    #[inline]
    fn write_false(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[0;33mfalse\x1b[0m")
        } else {
            out.write_str("false")
        }
    }

    #[inline]
    fn write_null(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[0;35mnull\x1b[0m")
        } else {
            out.write_str("null")
        }
    }

    #[inline]
    fn write_number(&self, s: &str, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[0;36m")?;
            out.write_str(s)?;
            out.write_str("\x1b[0m")
        } else {
            out.write_str(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::format::{BytePos, Color, Formatter};

    #[test]
    fn parse_number_ok() {
        let datas = [
            // Parse some integers
            ("1234xxxx", "1234"),
            ("42", "42"),
            (
                "1233456787766677889778998789988",
                "1233456787766677889778998789988",
            ),
            ("0000", "0"),
            ("0", "0"),
            ("-0", "-0"),
            ("012345", "0"),
            ("0abcdef", "0"),
            ("-10256", "-10256"),
            ("-012344", "-0"),
            // Parse real (with fraction)
            ("1.000", "1.000"),
            ("1.7b", "1.7"),
        ];
        for (input, expected) in datas {
            let mut formatter = Formatter::new(input.as_bytes(), Color::NoColor);
            let mut out = String::new();
            formatter.parse_number(&mut out).unwrap();
            assert_eq!(out, expected);
        }
    }

    #[test]
    fn parse_number_failed() {
        let datas = ["1.", "78980.a", "abc"];
        for input in datas {
            let mut formatter = Formatter::new(input.as_bytes(), Color::NoColor);
            let mut out = String::new();
            let result = formatter.parse_number(&mut out);
            assert!(result.is_err());
        }
    }

    fn assert_against_std(bytes: &[u8], len: usize) {
        // We pass the full buffer to the parser, with some trailing bytes
        let mut formatter = Formatter::new(&bytes, Color::NoColor);
        let ret = formatter.next_utf8_char();

        // We test against a buffer without trailing
        match std::str::from_utf8(&bytes[..len]) {
            Ok(str) => {
                assert!(ret.is_ok());
                assert_eq!(formatter.pos.0, len);
                let out = formatter.slice_str_unchecked(BytePos(0), formatter.pos);
                assert_eq!(out, str);
            }
            Err(_) => {
                assert!(ret.is_err());
            }
        }
    }

    #[test]
    fn try_read_one_byte_to_utf8() {
        // Iterate through all 1-byte UTF-8 bytes, even invalid
        for b in 0x00..=0xFF {
            let bytes = [b, b'x', b'x', b'x'];
            assert_against_std(&bytes, 1);
        }
    }

    #[test]
    fn try_read_two_bytes_to_utf8() {
        // Iterate through all UTF-8 2-bytes: C0..=DF 80..=BF
        // It may contains invalid ones (overlong for instance).
        for b1 in 0xC0..=0xDF {
            for b2 in 0x80..=0xBF {
                let bytes = [b1, b2, b'x', b'x', b'x'];
                assert_against_std(&bytes, 2);
            }
        }
    }

    #[test]
    fn try_read_three_bytes_to_utf8() {
        // Iterate through all UTF-8 3-bytes: E0..=EF 80..=BF 80..=BF
        // It may contains invalid ones (overlong for instance).
        for b1 in 0xF0..=0xF7 {
            for b2 in 0x80..=0xBF {
                for b3 in 0x80..=0xBF {
                    let bytes = [b1, b2, b3, b'x', b'x', b'x'];
                    assert_against_std(&bytes, 3);
                }
            }
        }
    }

    #[test]
    fn try_read_four_bytes_to_utf8() {
        // Iterate through all UTF-8 4-bytes: F0..=F7 80..=BF 80..=BF 80..=BF
        // It may contains invalid ones (overlong for instance).
        for b1 in 0xF0..=0xF7 {
            for b2 in 0x80..=0xBF {
                for b3 in 0x80..=0xBF {
                    for b4 in 0x80..=0xBF {
                        let bytes = [b1, b2, b3, b4, b'x', b'x', b'x'];
                        assert_against_std(&bytes, 4);
                    }
                }
            }
        }
    }

    #[test]
    fn format_demo_string() {
        let input = r#"{"strings":{"english":"Hello, world!","chinese":"你好，世界","japanese":"こんにちは世界","korean":"안녕하세요 세계","arabic":"مرحبا بالعالم","hindi":"नमस्ते दुनिया","russian":"Привет, мир","greek":"Γειά σου Κόσμε","hebrew":"שלום עולם","accented":"Curaçao, naïve, façade, jalapeño"},"numbers":{"zero":0,"positive_int":42,"negative_int":-42,"large_int":1234567890123456789,"small_float":0.000123,"negative_float":-3.14159,"large_float":1.7976931348623157e308,"smallest_float":5e-324,"sci_notation_positive":6.022e23,"sci_notation_negative":-2.99792458e8},"booleans":{"isActive":true,"isDeleted":false},"emojis":{"happy":"😀","sad":"😢","fire":"🔥","rocket":"🚀","earth":"🌍","heart":"❤️","multi":"👩‍💻🧑🏽‍🚀👨‍👩‍👧‍👦"},"nothing":null}"#;
        let mut formatter = Formatter::new(input.as_bytes(), Color::NoColor);
        let mut out = String::new();
        formatter.format(&mut out).unwrap();
        assert_eq!(out, r#"{
  "strings": {
    "english": "Hello, world!",
    "chinese": "你好，世界",
    "japanese": "こんにちは世界",
    "korean": "안녕하세요 세계",
    "arabic": "مرحبا بالعالم",
    "hindi": "नमस्ते दुनिया",
    "russian": "Привет, мир",
    "greek": "Γειά σου Κόσμε",
    "hebrew": "שלום עולם",
    "accented": "Curaçao, naïve, façade, jalapeño"
  },
  "numbers": {
    "zero": 0,
    "positive_int": 42,
    "negative_int": -42,
    "large_int": 1234567890123456789,
    "small_float": 0.000123,
    "negative_float": -3.14159,
    "large_float": 1.7976931348623157e308,
    "smallest_float": 5e-324,
    "sci_notation_positive": 6.022e23,
    "sci_notation_negative": -2.99792458e8
  },
  "booleans": {
    "isActive": true,
    "isDeleted": false
  },
  "emojis": {
    "happy": "😀",
    "sad": "😢",
    "fire": "🔥",
    "rocket": "🚀",
    "earth": "🌍",
    "heart": "❤️",
    "multi": "👩‍💻🧑🏽‍🚀👨‍👩‍👧‍👦"
  },
  "nothing": null
}"#)
    }
}
