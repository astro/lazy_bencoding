use bencoded::*;

pub struct ListIterator<'a> {
    data: &'a [u8]
}

impl<'a> ListIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        if data.len() >= 2 || data[0] as char == 'l' {
            ListIterator { data: &data[1..] }
        } else {
            ListIterator { data: b"" }
        }
    }
}


impl<'a> Iterator for ListIterator<'a> {
    type Item = BEncoded<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() < 1 || self.data[0] as char == 'e' {
            return None
        }

        let b = BEncoded::new(self.data);
        match b.this_and_rest() {
            None => None,
            Some((elem, rest)) => {
                self.data = rest;
                Some(elem)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use token::*;
    use super::*;

    fn parse_and_collect<'a>(data: &'a [u8]) -> Vec<Vec<Token<'a>>> {
        ListIterator::new(data)
            .map(|elem| {
                elem.collect()
            })
            .collect()
    }

    #[test]
    fn test_non_list() {
        let results = parse_and_collect(b"i23e");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_list1() {
        let results = parse_and_collect(b"l1:ai23e1:bi5ee");
        assert_eq!(results, [
            [::Token::ByteString(b"a")], [::Token::Integer(b"23")],
            [::Token::ByteString(b"b")], [::Token::Integer(b"5")],
        ]);
    }

    #[test]
    fn test_list2() {
        let results = parse_and_collect(b"ll1:ai23eed1:bi5eee");
        assert_eq!(results, [
            [::Token::ListStart, ::Token::ByteString(b"a"), ::Token::Integer(b"23"), ::Token::End],
            [::Token::DictStart, ::Token::ByteString(b"b"), ::Token::Integer(b"5"), ::Token::End],
        ]);
    }

    #[test]
    fn test_empty() {
        let results = parse_and_collect(b"le");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_trailing() {
        let results = parse_and_collect(b"lei23e");
        assert_eq!(results.len(), 0);
    }
}
