extern crate lazy_bencoding;

use std::env;
use std::fs::File;
use std::io::Read;

use lazy_bencoding::*;

fn pretty_print<'a>(bencoded: BEncoded<'a>, padding: &str) {
    if let Some(s) = bencoded.get_utf8_string() {
        println!("{}{}", padding, s);
    } else if let Some(bs) = bencoded.get_byte_string() {
        // Hexdump
        let mut line = String::new();
        for (i, b) in bs.iter().enumerate() {
            line.push_str(&format!("{:02X}", b));
            if i & 15 == 0 {
                println!("{}{}", padding, line);
                line.clear();
            } else if i & 7 == 0 {
                line.push_str("   ");
            } else if i & 3 == 0 {
                line.push_str("  ");
            } else {
                line.push_str(" ");
            }
        }
        if line.len() > 0 {
            println!("{}{}", padding, line);
        }
    } else if let Some(i) = bencoded.get_integer() {
        println!("{}{}", padding, i);
    } else if bencoded.is_dict() {
        for (key, value) in bencoded.dict() {
            if let Some(key) = key.get_utf8_string() {
                println!("{}{}:", padding, key);
            } else {
                pretty_print(key, padding);
            }
            pretty_print(value, &format!("{}    ", padding));
        }
    } else if bencoded.is_list() {
        for (i, elem) in bencoded.list().enumerate() {
            let padding = if i == 0 {
                format!("{}  - ", padding)
            } else {
                let space_padding: String = padding.chars()
                    .map(|c| ' ').collect();
                format!("{}  - ", space_padding)
            };
            pretty_print(elem, &padding);
        }
    } else {
        println!("{}Weird tokens: {:?}", padding, bencoded.collect::<Vec<Token<'a>>>());
    }
}

fn main() {
    for filename in env::args().skip(1) {
        let mut f = match File::open(&filename) {
            Ok(f) => f,
            Err(e) => {
                println!("{}: {:?}", filename, e);
                continue;
            }
        };
        let mut contents = Vec::new();
        f.read_to_end(&mut contents).unwrap();

        let bencoded = BEncoded::new(&contents[..]);
        pretty_print(bencoded, "");
    }
}
