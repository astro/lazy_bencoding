#[derive(Eq, PartialEq, Debug)]
pub enum Token<'a> {
    ByteString(&'a [u8]),
    Integer(&'a [u8]),
    ListStart,
    DictStart,
    End,
}
