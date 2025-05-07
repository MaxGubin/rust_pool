pub mod protocol;
pub mod serial;
pub mod message;
use std::sync::{Arc, RwLock};

pub type PoolProtocolRW = Arc<RwLock<protocol::PoolProtocol>>;
