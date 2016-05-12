use bencoded::*;

pub struct DictIterator<'a> {
    data: &'a [u8]
}

impl<'a> DictIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        if data.len() >= 2 || data[0] as char == 'd' {
            DictIterator { data: &data[1..] }
        } else {
            DictIterator { data: b"" }
        }
    }
}


impl<'a> Iterator for DictIterator<'a> {
    type Item = (BEncoded<'a>, BEncoded<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() < 1 || self.data[0] as char == 'e' {
            return None
        }

        let b = BEncoded::new(self.data);
        match b.this_and_rest() {
            None => None,
            Some((key, rest)) => {
                let b = BEncoded::new(rest);
                match b.this_and_rest() {
                    None => None,
                    Some((value, rest)) => {
                        self.data = rest;
                        Some((key, value))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use token::*;
    use super::*;

    fn parse_and_collect<'a>(data: &'a [u8]) -> Vec<(Vec<Token<'a>>, Vec<Token<'a>>)> {
        DictIterator::new(data)
            .map(|(key, value)| {
                (key.collect(), value.collect())
            })
            .collect()
    }

    #[test]
    fn test_non_dict() {
        let results = parse_and_collect(b"i23e");
        assert_eq!(results, []);
    }

    #[test]
    fn test_dict1() {
        let results = parse_and_collect(b"d1:ai23e1:bi5ee");
        assert_eq!(results, [
            (vec![::Token::ByteString(b"a")], vec![::Token::Integer(b"23")]),
            (vec![::Token::ByteString(b"b")], vec![::Token::Integer(b"5")]),
        ]);
    }

    #[test]
    fn test_empty() {
        let results = parse_and_collect(b"de");
        assert_eq!(results, []);
    }

    #[test]
    fn test_trailing() {
        let results = parse_and_collect(b"dei23e");
        assert_eq!(results, []);
    }
}
