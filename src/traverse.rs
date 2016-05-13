use std::str::from_utf8;
use bencoded::*;
use parse::*;
use token::*;
use dict::*;
use list::*;

impl<'a> BEncoded<'a> {
    /// Is the remaining `data` a dict?
    pub fn is_dict(&self) -> bool {
        match parse_token(self.data) {
            Some((Token::DictStart, _)) => true,
            _ => false,
        }
    }

    /// Is the remaining `data` a list?
    pub fn is_list(&self) -> bool {
        match parse_token(self.data) {
            Some((Token::ListStart, _)) => true,
            _ => false,
        }
    }

    /// Is the remaining `data` a string? Then get it as a cheap slice.
    pub fn get_byte_string(&self) -> Option<&'a [u8]> {
        match parse_token(self.data) {
            Some((Token::ByteString(s), _)) => Some(s),
            _ => None,
        }
    }

    /// Is the remaining `data` a string? Then get it as checked UTF-8.
    pub fn get_utf8_string(&self) -> Option<&'a str> {
        self.get_byte_string()
            .and_then(|s| match from_utf8(s) {
                Ok(s) => Some(s),
                Err(_) => None,
            })
    }

    /// Is the remaining `data` an integer? If so, parse and return it.
    pub fn get_integer(&self) -> Option<i64> {
        let s = match parse_token(self.data) {
            Some((Token::Integer(bs), _)) => bs,
            _ => return None
        };

        let mut sig = 1;
        let mut position = 0;
        match s[0] as char {
            '-' => {
                sig = -1;
                position += 1;
            },
            _ => (),
        }

        let mut value = 0;
        for ch in &s[position..] {
            match *ch as char {
                '0'...'9' =>
                    value = (value * 10) + *ch as i64 - '0' as i64,
                _ =>
                    return None
            }
        }
        Some(sig * value)
    }

    /// Used by [`dict()`](#method.dict) and [`list()`](#method.list)
    ///
    /// Restricts the `BEncoded<'a>` to a single item (string,
    /// integer, list, or dict). Also returns the trailing data.
    pub fn this_and_rest(mut self) -> Option<(BEncoded<'a>, &'a [u8])> {
        let data = &self.data[..];
        while self.depth >= 0 {
            match self.next() {
                None => return None,
                _ => (),
            }
        }

        let this = BEncoded::new(&data[0..(data.len() - self.data.len())]);
        Some((this, self.data))
    }

    /// If the remaining data is a list, return value fulfills
    /// `Iterator<Item=BEncoded<'a>>`
    pub fn list(&self) -> ListIterator {
        ListIterator::new(self.data)
    }

    /// If the remaining data is a dict, return value fulfills
    /// `Iterator<Item=(BEncoded<'a>, BEncoded<'a>)>`
    pub fn dict(&self) -> DictIterator {
        DictIterator::new(self.data)
    }

    /// From a [`dict()`](#method.dict), get the value of key
    /// `wanted_key`.
    ///
    /// In case the key is not present, we return an empty
    /// instance. This allows chaining these calls conveniently:
    ///
    /// ```
    /// # use self::lazy_bencoding::*;
    /// let bencoded = BEncoded::new(b"d1:ad1:ad1:a4:spame1:bd1:a4:eggseee");
    /// assert_eq!(bencoded.get(b"a").get(b"b").get(b"a").get_utf8_string(),
    ///            Some("eggs"));
    /// ```
    pub fn get(&'a self, wanted_key: &'a [u8]) -> Self {
        for (mut key, value) in self.dict() {
            match (key.next(), key.next()) {
                (Some(Token::ByteString(s)), None) if s == wanted_key => return value,
                _ => (),
            }
        }

        // Default value:
        BEncoded::empty()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_dict() {
        let bencoded = ::BEncoded::new(b"de");
        assert!(bencoded.is_dict(), "is_dict");
    }

    #[test]
    fn test_is_list() {
        let bencoded = ::BEncoded::new(b"le");
        assert!(bencoded.is_list(), "is_list");
    }

    #[test]
    fn test_get_byte_string() {
        let bencoded = ::BEncoded::new(b"4:spam");
        let expected: &'static [u8] = b"spam";
        assert_eq!(bencoded.get_byte_string(), Some(expected));
    }

    #[test]
    fn test_get_utf8_string() {
        let bencoded = ::BEncoded::new(b"4:spam");
        assert_eq!(bencoded.get_utf8_string(), Some("spam"));
    }

    #[test]
    fn test_get_integer() {
        let bencoded = ::BEncoded::new(b"i23e");
        assert_eq!(bencoded.get_integer(), Some(23));
    }

    #[test]
    fn test_get_negative_integer() {
        let bencoded = ::BEncoded::new(b"i-5e");
        assert_eq!(bencoded.get_integer(), Some(-5));
    }

    #[test]
    fn test_this_and_rest1() {
        let bencoded = ::BEncoded::new(b"4:spam4:eggs");
        let result = bencoded.this_and_rest().map(|(bencoded, rest)| (bencoded.data, rest));

        let expected_this: &'static [u8] = b"4:spam";
        let expected_rest: &'static [u8] = b"4:eggs";
        assert_eq!(result, Some((expected_this, expected_rest)));
    }

    #[test]
    fn test_this_and_rest2() {
        let bencoded = ::BEncoded::new(b"l4:spam4:eggseli23ee");
        let result = bencoded.this_and_rest().map(|(bencoded, rest)| (bencoded.data, rest));

        let expected_this: &'static [u8] = b"l4:spam4:eggse";
        let expected_rest: &'static [u8] = b"li23ee";
        assert_eq!(result, Some((expected_this, expected_rest)));
    }

    #[test]
    fn test_get_key_not_found() {
        let bencoded = ::BEncoded::new(b"d1:a4:spam1:b4:eggs1:ci42ee");
        assert_eq!(bencoded.get(b"x").data, b"");
    }

    #[test]
    fn test_get_key1() {
        let bencoded = ::BEncoded::new(b"d1:a4:spame");
        let a_bencoded = bencoded.get(b"a");
        let mut a = Vec::new();
        for token in a_bencoded {
            a.push(token);
        }
        assert_eq!(a, vec![::Token::ByteString(b"spam")]);
    }

    #[test]
    fn test_get_key2() {
        let bencoded = ::BEncoded::new(b"d1:a4:spam1:b4:eggs1:ci42ee");
        assert_eq!(bencoded.get(b"a").get_utf8_string(), Some("spam"));
        assert_eq!(bencoded.get(b"b").get_utf8_string(), Some("eggs"));
        assert_eq!(bencoded.get(b"c").get_integer(), Some(42));
    }

    #[test]
    fn test_get_key_nested() {
        let bencoded = ::BEncoded::new(b"d1:ad1:ad1:a4:spame1:bd1:a4:eggseee");
        assert_eq!(bencoded.get(b"a").get(b"a").get(b"a").get_utf8_string(),
                   Some("spam"));
        assert_eq!(bencoded.get(b"a").get(b"b").get(b"a").get_utf8_string(),
                   Some("eggs"));
    }
}
