
use serde::{Deserialize, Serialize};
// Controller structure.

#[derive(Serialize, Deserialize, Debug)]
pub struct Comms {
    
    c_type: String,
    portId: u32,
    enabled: bool,
    rs485Port: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackupInterval {
    days: u32,
    hours: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Backup {
    automatic: bool,
    interval: BackupInterval,
    keepCount: u32,
    njsPC: bool, //?
    servers: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Controller {
    comms: Comms,
    backup: Backup,

}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            comms: Comms {
                c_type: String::from("RS485"),
                portId: 0,
                enabled: true,
                rs485Port: String::from("/dev/ttyUSB0"),
            },
            backup: Backup {
                automatic: false,
                interval: BackupInterval {
                    days: 40,
                    hours: 0,
                },
                keepCount: 0,
                njsPC: false,
                servers: Vec::new(),
            },
        }
    }
}