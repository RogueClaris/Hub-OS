// Increment VERSION_ITERATION lib.rs if packets are added or modified

use crate::structures::{
    FileHash, Input, InstalledBlock, InstalledSwitchDrive, PackageCategory, PackageId,
};
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum NetplaySignal {
    AttemptingFlee,
    CompletedFlee,
    Disconnect,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct NetplayBufferItem {
    pub pressed: Vec<Input>,
    pub signals: Vec<NetplaySignal>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetplayPacket {
    pub index: usize,
    pub data: NetplayPacketData,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, IntoStaticStr)]
pub enum NetplayPacketData {
    Heartbeat,
    Hello,
    HelloAck,
    PlayerSetup {
        player_package: PackageId,
        script_enabled: bool,
        // package_id, code
        cards: Vec<(PackageId, String)>,
        regular_card: Option<usize>,
        recipes: Vec<PackageId>,
        blocks: Vec<InstalledBlock>,
        drives: Vec<InstalledSwitchDrive>,
    },
    PackageList {
        // category, package_id, hash
        packages: Vec<(PackageCategory, PackageId, FileHash)>,
    },
    MissingPackages {
        recipient_index: usize,
        list: Vec<FileHash>,
    },
    ReadyForPackages,
    PackageZip {
        data: Vec<u8>,
    },
    Ready,
    Buffer {
        data: NetplayBufferItem,
        lead: Vec<i16>,
    },
}

impl NetplayPacket {
    pub fn new_disconnect_signal(index: usize) -> NetplayPacket {
        NetplayPacket {
            index,
            data: NetplayPacketData::Buffer {
                data: NetplayBufferItem {
                    pressed: Vec::new(),
                    signals: vec![NetplaySignal::Disconnect],
                },
                lead: Vec::new(),
            },
        }
    }
}
