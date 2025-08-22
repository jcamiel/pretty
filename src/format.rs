use std::cmp::PartialEq;
use std::fmt;
use std::fmt::Write;

/// A fast JSON formatter / pretty printer.
/// This is a fast JSON formatter (x2 compared to pretty printing with [Serde JSON](https://github.com/serde-rs/json)).
/// This parser processes byte by byte and do not require pre UTF-8 validation. UTF-8 validation
/// is done on the fly, while parsing strings. This implementation try to not allocate anything.
/// It does not try to normalise, remove unnecessary escaping, it just formats the actual input
/// with spaces and (optionally) color.
pub struct Parser<'input> {
    input: &'input [u8],
    pos: BytePos,
    indent: usize,
    /// Use color with ANSI escape code when prettifying.
    color: Color,
}

/// A byte position in a bytes stream.
#[derive(Debug, Copy, Clone)]
struct BytePos(usize);

#[derive(Debug)]
pub enum ParseError {
    Eof,
    InvalidByte(u8),
    InvalidUtf8,
    InvalidEscape(u8),
    Fmt(fmt::Error),
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

type ParseResult<T> = Result<T, ParseError>;

impl From<fmt::Error> for ParseError {
    fn from(e: fmt::Error) -> Self {
        ParseError::Fmt(e)
    }
}

impl<'input> Parser<'input> {
    pub fn new(input: &'input [u8], color: Color) -> Self {
        Parser {
            input,
            pos: BytePos(0),
            indent: 0,
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
    fn expect_byte(&mut self, expected: u8) -> ParseResult<()> {
        match self.next_byte() {
            Some(b) if b == expected => Ok(()),
            Some(b) => Err(ParseError::InvalidByte(b)),
            None => Err(ParseError::Eof),
        }
    }

    /// Format and prettify the JSON
    pub fn format(&mut self, out: &mut impl Write) -> ParseResult<()> {
        self.skip_whitespace();
        self.parse_value(out)?;
        self.skip_whitespace();

        // Have we completely consumed our payload?
        if let Some(b) = self.peek_byte() {
            Err(ParseError::InvalidByte(b))
        } else {
            Ok(())
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos.0 += 1;
        }
    }

    /// Value
    fn parse_value(&mut self, out: &mut impl Write) -> ParseResult<()> {
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
            Some(b) => Err(ParseError::InvalidByte(b)),
            None => Err(ParseError::Eof),
        }
    }

    /// Object
    fn parse_object(&mut self, out: &mut impl Write) -> ParseResult<()> {
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
        self.indent += 1;

        let mut first = true;
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.next_byte();
                self.indent -= 1;
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
    fn parse_array(&mut self, out: &mut impl Write) -> ParseResult<()> {
        // From <https://datatracker.ietf.org/doc/html/rfc7159#section-4>:
        // array = begin-array [ value *( value-separator value ) ] end-array
        self.expect_byte(b'[')?;
        self.write_begin_arr(out)?;
        self.indent += 1;

        let mut first = true;
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b']') {
                self.next_byte();
                self.indent -= 1;
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
    fn parse_string(&mut self, out: &mut impl Write, mode: StringMode) -> ParseResult<()> {
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
                                let hex = self.next_byte().ok_or(ParseError::Eof)?;
                                if !(hex as char).is_ascii_hexdigit() {
                                    return Err(ParseError::InvalidByte(hex));
                                }
                            }
                        }
                        Some(b) => return Err(ParseError::InvalidEscape(b)),
                        None => return Err(ParseError::Eof),
                    }
                }
                0x00..=0x1F => return Err(ParseError::InvalidByte(b)),
                _ => {
                    // Decode valid UTF-8 char
                    self.next_utf8_char()?;
                }
            }
        }
        Err(ParseError::Eof)
    }

    /// Literals
    fn parse_true(&mut self, out: &mut impl Write) -> ParseResult<()> {
        for &b in b"true" {
            self.expect_byte(b)?;
        }
        self.write_true(out)?;
        Ok(())
    }

    fn parse_false(&mut self, out: &mut impl Write) -> ParseResult<()> {
        for &b in b"false" {
            self.expect_byte(b)?;
        }
        self.write_false(out)?;
        Ok(())
    }

    fn parse_null(&mut self, out: &mut impl Write) -> ParseResult<()> {
        for &b in b"null" {
            self.expect_byte(b)?;
        }
        self.write_null(out)?;
        Ok(())
    }

    /// Parse a JSON number.
    fn parse_number(&mut self, out: &mut impl Write) -> ParseResult<()> {
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

    fn parse_integer(&mut self) -> ParseResult<()> {
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
            Some(b) => Err(ParseError::InvalidByte(b)),
            None => Err(ParseError::Eof),
        }
    }

    fn parse_fraction(&mut self) -> ParseResult<()> {
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
                Some(b) => Err(ParseError::InvalidByte(b)),
                None => Err(ParseError::Eof),
            }?
        }
        Ok(())
    }

    fn parse_exponent(&mut self) -> ParseResult<()> {
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
                    Some(b) => Err(ParseError::InvalidByte(b)),
                    None => Err(ParseError::Eof),
                }
            }
            _ => Ok(()),
        }
    }

    // -------- UTF-8 decoder --------
    fn next_utf8_char(&mut self) -> ParseResult<()> {
        #[inline(always)]
        fn cont(b: u8) -> bool {
            (b & 0xC0) == 0x80
        }

        let b1 = self.next_byte().ok_or(ParseError::Eof)?;
        if b1 < 0x80 {
            return Ok(());
        }

        let b2 = self.next_byte().ok_or(ParseError::Eof)?;
        if b1 < 0xE0 {
            return if (0xC2..=0xDF).contains(&b1) && cont(b2) {
                Ok(())
            } else {
                Err(ParseError::InvalidUtf8)
            };
        }

        let b3 = self.next_byte().ok_or(ParseError::Eof)?;
        if b1 < 0xF0 {
            return if match b1 {
                0xE0 => (0xA0..=0xBF).contains(&b2) && cont(b3),
                0xED => (0x80..=0x9F).contains(&b2) && cont(b3), // no surrogates
                0xE1..=0xEC | 0xEE..=0xEF => cont(b2) && cont(b3),
                _ => false,
            } {
                Ok(())
            } else {
                Err(ParseError::InvalidUtf8)
            };
        }

        let b4 = self.next_byte().ok_or(ParseError::Eof)?;
        if match b1 {
            0xF0 => (0x90..=0xBF).contains(&b2) && cont(b3) && cont(b4),
            0xF4 => (0x80..=0x8F).contains(&b2) && cont(b3) && cont(b4),
            0xF1..=0xF3 => cont(b2) && cont(b3) && cont(b4),
            _ => false,
        } {
            Ok(())
        } else {
            Err(ParseError::InvalidUtf8)
        }
    }
}

const SPACES: &str = "                                                                 ";


/// Methods to print on a [Write], with color, or not.
impl<'input> Parser<'input> {
    fn write_indent(&self, out: &mut impl Write) -> Result<(), fmt::Error> {
        let n = self.indent * 2;
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
            out.write_str("\x1b[1;39m]\x1b[0m\n")
        } else {
            out.write_str("]\n")
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
            out.write_str("null")
        } else {
            out.write_str("null")
        }
    }

    #[inline]
    fn write_number(&self, s: &str, out: &mut impl Write) -> Result<(), fmt::Error> {
        if self.color == Color::AnsiCode {
            out.write_str("\x1b[0;35m")?;
            out.write_str(s)?;
            out.write_str("\x1b[0m")
        } else {
            out.write_str(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::format::{BytePos, Color, Parser};

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
            let mut parser = Parser::new(input.as_bytes(), Color::NoColor);
            let mut out = String::new();
            parser.parse_number(&mut out).unwrap();
            assert_eq!(out, expected);
        }
    }

    #[test]
    fn parse_number_failed() {
        let datas = ["1.", "78980.a", "abc"];
        for input in datas {
            let mut parser = Parser::new(input.as_bytes(), Color::NoColor);
            let mut out = String::new();
            let result = parser.parse_number(&mut out);
            assert!(result.is_err());
        }
    }

    fn assert_against_std(bytes: &[u8], len: usize) {
        // We pass the full buffer to the parser, with some trailing bytes
        let mut parser = Parser::new(&bytes, Color::NoColor);
        let ret = parser.next_utf8_char();

        // We test against a buffer without trailing
        match std::str::from_utf8(&bytes[..len]) {
            Ok(str) => {
                assert!(ret.is_ok());
                assert_eq!(parser.pos.0, len);
                let out = parser.slice_str_unchecked(BytePos(0), parser.pos);
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
}
