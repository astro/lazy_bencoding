use token::*;

pub fn parse_token<'a>(data: &'a [u8]) -> Option<(Token<'a>, &'a [u8])> {
    if data.len() < 1 {
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
            let rest = &data[(pos + 1)..];
            Some((token, rest))
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
                let rest = &data[(pos + len)..];
                Some((Token::ByteString(buf), rest))
            } else {
                // Not enough data for this string
                None
            }
        }
        'l' => {
            let rest = &data[1..];
            Some((Token::ListStart, rest))
        }
        'd' => {
            let rest = &data[1..];
            Some((Token::DictStart, rest))
        }
        'e' => {
            let rest = &data[1..];
            Some((Token::End, rest))
        }
        _ => None,  // Unexpected: terminate
    }
}
