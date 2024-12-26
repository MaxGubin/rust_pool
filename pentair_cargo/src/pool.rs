pub mod protocol;
pub mod serial;
use std::sync::{Arc, RwLock};

pub type PoolProtocolRW = Arc<RwLock<protocol::PoolProtocol>>;
