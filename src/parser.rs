use std::fmt;
use std::fmt::Write;

pub struct Parser<'input> {
    input: &'input [u8],
    pos: usize,
    indent: usize,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ParseError {
    Eof,
    InvalidByte(u8),
    InvalidUtf8,
    InvalidEscape(u8),
    Fmt(fmt::Error),
}

type ParseResult<T> = Result<T, ParseError>;

impl From<fmt::Error> for ParseError {
    fn from(e: fmt::Error) -> Self {
        ParseError::Fmt(e)
    }
}

const SPACES: &str = "                                                                 ";

impl<'input> Parser<'input> {
    pub fn new(input: &'input [u8]) -> Self {
        Parser {
            input,
            pos: 0,
            indent: 0,
        }
    }

    #[inline]
    fn next_byte(&mut self) -> Option<u8> {
        let b = self.peek_byte()?;
        self.pos += 1;
        Some(b)
    }

    #[inline]
    fn peek_byte(&mut self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    #[inline]
    fn expect_byte(&mut self, expected: u8) -> ParseResult<()> {
        match self.next_byte() {
            Some(b) if b == expected => Ok(()),
            Some(b) => Err(ParseError::InvalidByte(b)),
            None => Err(ParseError::Eof),
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

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

    // -------- Top-level parse --------
    pub fn parse(&mut self, out: &mut impl Write) -> ParseResult<()> {
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

    // -------- Value parsing --------
    fn parse_value(&mut self, out: &mut impl Write) -> ParseResult<()> {
        match self.peek_byte() {
            Some(b'"') => self.parse_string(out),
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

    // -------- Object --------
    fn parse_object(&mut self, out: &mut impl Write) -> ParseResult<()> {
        self.expect_byte(b'{')?;
        out.write_char('{')?;
        out.write_char('\n')?;
        self.indent += 1;

        let mut first = true;
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.next_byte();
                self.indent -= 1;
                self.write_indent(out)?;
                out.write_char('}')?;
                return Ok(());
            }

            if first {
                first = false;
            } else {
                self.expect_byte(b',')?;
                self.skip_whitespace();
                out.write_str(",\n")?;
            }

            // Parse key
            self.write_indent(out)?;
            self.parse_string(out)?;

            // Parse colon
            self.skip_whitespace();
            self.expect_byte(b':')?;
            out.write_str(": ")?;

            // Parse value
            self.skip_whitespace();
            self.parse_value(out)?;
        }
    }

    // -------- Array --------
    fn parse_array(&mut self, out: &mut impl Write) -> ParseResult<()> {
        self.expect_byte(b'[')?;
        out.write_str("[\n")?;
        self.indent += 1;

        let mut first = true;
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b']') {
                self.next_byte();
                self.indent -= 1;
                self.write_indent(out)?;
                out.write_char(']')?;
                return Ok(());
            }

            if first {
                first = false;
            } else {
                self.expect_byte(b',')?;
                self.skip_whitespace();
                out.write_str(",\n")?;
            }

            self.write_indent(out)?;
            self.parse_value(out)?;
        }
    }

    fn slice_str_unchecked(&self, start: usize, end: usize) -> &str {
        debug_assert!(start <= end && end <= self.input.len());
        let bytes = &self.input[start..end];
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    // -------- String (preserves escapes) --------
    fn parse_string(&mut self, out: &mut impl Write) -> ParseResult<()> {
        let start = self.pos;
        self.expect_byte(b'"')?;

        while let Some(b) = self.peek_byte() {
            match b {
                b'"' => {
                    self.next_byte();
                    // Flush plain segment before exit.
                    let string = self.slice_str_unchecked(start, self.pos);
                    out.write_str(string)?;
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

    // -------- Literals --------
    fn parse_true(&mut self, out: &mut impl Write) -> ParseResult<()> {
        for &b in b"true" {
            self.expect_byte(b)?;
        }
        out.write_str("true")?;
        Ok(())
    }

    fn parse_false(&mut self, out: &mut impl Write) -> ParseResult<()> {
        for &b in b"false" {
            self.expect_byte(b)?;
        }
        out.write_str("false")?;
        Ok(())
    }

    fn parse_null(&mut self, out: &mut impl Write) -> ParseResult<()> {
        for &b in b"null" {
            self.expect_byte(b)?;
        }
        out.write_str("null")?;
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
        out.write_str(digits)?;

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
    fn next_utf8_char(&mut self) -> ParseResult<char> {
        let b1 = self.next_byte().ok_or(ParseError::Eof)?;
        if b1 < 0x80 {
            return Ok(b1 as char);
        }
        let (needed, mut code): (usize, u32) = if b1 & 0b1110_0000 == 0b1100_0000 {
            (2, (b1 & 0b0001_1111) as u32)
        } else if b1 & 0b1111_0000 == 0b1110_0000 {
            (3, (b1 & 0b0000_1111) as u32)
        } else if b1 & 0b1111_1000 == 0b1111_0000 {
            (4, (b1 & 0b0000_0111) as u32)
        } else {
            return Err(ParseError::InvalidUtf8);
        };

        for _ in 1..needed {
            let b = self.next_byte().ok_or(ParseError::Eof)?;
            if b & 0b1100_0000 != 0b1000_0000 {
                return Err(ParseError::InvalidUtf8);
            }
            code = (code << 6) | (b & 0b0011_1111) as u32;
        }
        // TODO: Reject surrogate halves?
        // JSON requires UTF-8 validity (no overlong encodings, no surrogate halves).
        // Right now, \xED\xA0\x80 (UTF-8 surrogate) will be accepted
        // if (0xD800..=0xDFFF).contains(&code) {
        //     return Err(ParseError::InvalidUtf8);
        // }
        char::from_u32(code).ok_or(ParseError::InvalidUtf8)
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;

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
            let mut parser = Parser::new(input.as_bytes());
            let mut out = String::new();
            parser.parse_number(&mut out).unwrap();
            assert_eq!(out, expected);
        }
    }

    #[test]
    fn parse_number_failed() {
        let datas = ["1.", "78980.a", "abc"];
        for input in datas {
            let mut parser = Parser::new(input.as_bytes());
            let mut out = String::new();
            let result = parser.parse_number(&mut out);
            assert!(result.is_err());
        }
    }
}
