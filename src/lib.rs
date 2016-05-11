pub struct BEncoded {
    data: Vec<u8>
}

impl BEncoded {
    pub fn new(data: Vec<u8>) -> Self {
        BEncoded {
            data: data
        }
    }
}

pub struct BEncodedParser<'a> {
    bencoded: &'a BEncoded,
    position: usize
}

impl<'a> BEncodedParser<'a> {
    fn read_integer(&mut self) -> u64 {
        let mut value = 0;
        let data = &self.bencoded.data;

        loop {
            let ch = data[self.position] as char;
            if ch >= '0' && ch <= '9' {
                value = value * 10 + (ch as u64 - '0' as u64);
                self.position += 1;
            } else {
                break
            }
        }
        value
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum Token<'a> {
    ByteString(&'a [u8]),
    Integer(i64),
    ListStart,
    DictStart,
    End
}

impl<'a> IntoIterator for &'a BEncoded {
    type Item = Token<'a>;
    type IntoIter = BEncodedParser<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BEncodedParser {
            bencoded: self,
            position: 0
        }
    }
}

impl<'a> Iterator for BEncodedParser<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let data = &self.bencoded.data;
        if self.position >= data.len() {
            // Already done
            return None
        }

        match data[self.position] as char {
            'i' => {
                self.position += 1;

                let sig = if data[self.position] as char == '-' {
                    self.position += 1;
                    -1
                } else {
                    1
                };
                let value = self.read_integer();
                if data[self.position] as char == 'e' {
                    self.position += 1;
                    Some(Token::Integer(sig * value as i64))
                } else {
                    // Error: terminate
                    None
                }
            },
            '0'...'9' => {
                let len = self.read_integer() as usize;
                if data[self.position] as char == ':' && self.position + 1 + len <= data.len() {
                    self.position += 1;
                    let str = &data[self.position..(self.position + len)];
                    self.position += len;
                    Some(Token::ByteString(str))
                } else {
                    None
                }
            },
            'l' => {
                self.position += 1;
                Some(Token::ListStart)
            },
            'd' => {
                self.position += 1;
                Some(Token::DictStart)
            },
            'e' => {
                self.position += 1;
                Some(Token::End)
            },
            _ => None  // Unexpected: terminate
        }
    }
}

#[cfg(test)]
mod tests {
    fn expect(input: &'static [u8], expected: &[::Token<'static>]) {
        let mut input_buf = Vec::with_capacity(input.len());
        input_buf.extend_from_slice(input);
        let benc = ::BEncoded::new(input_buf);

        let mut result = Vec::with_capacity(expected.len());
        for token in &benc {
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
        expect(b"i3e", &[::Token::Integer(3)]);
        expect(b"i-3e", &[::Token::Integer(-3)]);
        expect(b"i0e", &[::Token::Integer(0)]);
    }

    #[test]
    fn test_lists() {
        expect(b"l4:spam4:eggse", &[
            ::Token::ListStart,
            ::Token::ByteString(b"spam"),
            ::Token::ByteString(b"eggs"),
            ::Token::End
        ]);
    }

    #[test]
    fn test_dict() {
        expect(b"d3:cow3:moo4:spam4:eggse", &[
            ::Token::DictStart,
            ::Token::ByteString(b"cow"),
            ::Token::ByteString(b"moo"),
            ::Token::ByteString(b"spam"),
            ::Token::ByteString(b"eggs"),
            ::Token::End
        ]);
        expect(b"d4:spaml1:a1:bee", &[
            ::Token::DictStart,
            ::Token::ByteString(b"spam"),
            ::Token::ListStart,
            ::Token::ByteString(b"a"),
            ::Token::ByteString(b"b"),
            ::Token::End,
            ::Token::End
        ]);
    }
}
