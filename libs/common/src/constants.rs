pub const SUM_OF_INFO_CELLS: u8 = 12;
pub const INDEX_STATE_CELL_DATA_LEN: usize = 1 + 1; // index(u8) + length(u8)

pub const INFO_CELL_DATA_LEN: usize = 1 + 1 + 8; // index(u8)| data_type(u8) |content(u64)
pub const INFO_CELL_META_LEN: usize = 1 + 1; // index(u8) | type(u8)
pub const INFO_CELL_CONTENT_LEN: usize = INFO_CELL_DATA_LEN - INFO_CELL_META_LEN; // u64
pub const INFO_CELL_META_TYPE_POS: usize = 1; // second byte is content type

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum DataType {
    Arbitrage = 0,
    Timestamp = 1,
    BlockNumber = 2,
}
