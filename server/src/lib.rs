use nanoserde::{DeBin, SerBin};

#[derive(DeBin, SerBin, PartialEq, Debug)]
pub enum MetaMessage {
    ConnectionRequest,
    OpponentIp {
        id: i32,
        ip: String
    },
}
