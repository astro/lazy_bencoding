/// Token types of BEncoding
///
/// Returned by [`impl<'a> Iterator for BEncoded<'a>`](struct.BEncoded.html#associatedtype.Item)
#[derive(Eq, PartialEq, Debug)]
pub enum Token<'a> {
    // Signified by preceding length, eg. `4:spam`
    ByteString(&'a [u8]),
    // Signified by leading `i` and trailing `e`, eg. `i-23e`
    Integer(&'a [u8]),
    // `l`
    ListStart,
    // `d`
    DictStart,
    // `e` for both lists and dicts
    End,
}
