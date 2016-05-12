use token::*;
use parse::*;


#[derive(Clone, Copy)]
pub struct BEncoded<'a> {
    pub data: &'a [u8],
    pub depth: i16,
}

impl<'a> BEncoded<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BEncoded {
            data: data,
            depth: 0,
        }
    }

    pub fn empty() -> BEncoded<'static> {
        BEncoded {
            data: b"",
            depth: 0
        }
    }
}

impl<'a> Iterator for BEncoded<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.depth < 0 {
            // Already done
            return None;
        }

        match parse_token(self.data) {
            None => None,
            Some((token, rest)) => {
                match token {
                    Token::ListStart | Token::DictStart =>
                        self.depth += 1,
                    Token::End =>
                        self.depth -= 1,
                    _ =>
                        ()
                }
                if self.depth == 0 {
                    // Emit only one item at level 0
                    self.depth -= 1;
                }

                self.data = rest;
                Some(token)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    fn expect<'a>(input: &'a [u8], expected: &[::Token<'a>]) {
        let bencoded = ::BEncoded::new(input);
        let result = bencoded.collect::<Vec<::Token<'a>>>();
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

    #[test]
    fn test_only_one() {
        expect(b"4:spam4:eggs", &[::Token::ByteString(b"spam")]);
    }
}
