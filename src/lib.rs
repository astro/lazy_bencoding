pub struct BEncoded<'a> {
    data: &'a [u8],
    finished: bool,
}

impl<'a> BEncoded<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BEncoded {
            data: data,
            finished: false,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum Token<'a> {
    ByteString(&'a [u8]),
    Integer(&'a [u8]),
    ListStart,
    DictStart,
    End,
}

impl<'a> Iterator for BEncoded<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.data;
        if data.len() < 1 || self.finished {
            // Already done
            return None;
        }

        match data[0] as char {
            'i' => {
                let mut pos = 1;
                while data[pos] as char != 'e' {
                    pos += 1;
                }
                let token = Token::Integer(&data[1..pos]);
                self.data = &self.data[(pos + 1)..];
                Some(token)
            }
            ch @ '0'...'9' => {
                let mut len = ch as usize - '0' as usize;
                let mut pos = 1;
                while pos < data.len() {
                    let ch = data[pos] as char;
                    pos += 1;
                    if ch >= '0' && ch <= '9' {
                        len = 10 * len + ch as usize - '0' as usize;
                    } else if ch == ':' {
                        break;
                    } else {
                        // Error
                        return None;
                    }
                }

                if pos + len <= data.len() {
                    let buf = &data[pos..(pos + len)];
                    self.data = &data[(pos + len)..];
                    Some(Token::ByteString(buf))
                } else {
                    // Not enough data for this string
                    None
                }
            }
            'l' => {
                self.data = &self.data[1..];
                Some(Token::ListStart)
            }
            'd' => {
                self.data = &self.data[1..];
                Some(Token::DictStart)
            }
            'e' => {
                self.data = &self.data[1..];
                Some(Token::End)
            }
            _ => None,  // Unexpected: terminate
        }
    }
}

#[cfg(test)]
mod tests {
    fn expect(input: &'static [u8], expected: &[::Token<'static>]) {
        let benc = ::BEncoded::new(input);

        let mut result = Vec::with_capacity(expected.len());
        for token in benc {
            result.push(token);
        }
        assert_eq!(result, expected);
    }

    #[test]
    fn test_strings() {
        expect(b"4:spam", &[::Token::ByteString(b"spam")]);
    }

    #[test]
    fn test_integers() {
        expect(b"i3e", &[::Token::Integer(b"3")]);
        expect(b"i-3e", &[::Token::Integer(b"-3")]);
        expect(b"i0e", &[::Token::Integer(b"0")]);
    }

    #[test]
    fn test_lists() {
        expect(b"l4:spam4:eggse",
               &[::Token::ListStart,
                 ::Token::ByteString(b"spam"),
                 ::Token::ByteString(b"eggs"),
                 ::Token::End]);
    }

    #[test]
    fn test_dict() {
        expect(b"d3:cow3:moo4:spam4:eggse",
               &[::Token::DictStart,
                 ::Token::ByteString(b"cow"),
                 ::Token::ByteString(b"moo"),
                 ::Token::ByteString(b"spam"),
                 ::Token::ByteString(b"eggs"),
                 ::Token::End]);
        expect(b"d4:spaml1:a1:bee",
               &[::Token::DictStart,
                 ::Token::ByteString(b"spam"),
                 ::Token::ListStart,
                 ::Token::ByteString(b"a"),
                 ::Token::ByteString(b"b"),
                 ::Token::End,
                 ::Token::End]);
    }
}
