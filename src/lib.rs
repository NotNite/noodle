// You know what you're doing
#![allow(clippy::missing_safety_doc)]

use lazy_static::lazy_static;
use skidscan::{signature, Signature};
use std::path::Path;

pub type SharedSize = unsafe extern "C" fn(i32) -> i32;
pub type SharedSetWindow = unsafe extern "C" fn(*mut u8, i32, *mut u8, i32);
pub type TCPStateSize = unsafe extern "C" fn() -> i32;
pub type TCPTrain = unsafe extern "C" fn(*mut u8, *mut u8, *mut u8, *mut u8, i32);
pub type TCPDecode = unsafe extern "C" fn(*mut u8, *mut u8, *mut u8, i32, *mut u8, i32) -> bool;
pub type TCPEncode = unsafe extern "C" fn(*mut u8, *mut u8, *mut u8, i32, *mut u8, i32) -> bool;

lazy_static! {
    // https://github.com/ravahn/machina/blob/master/Machina.FFXIV/Memory/SigScan.cs
    static ref SHARED_SIZE: Signature = signature!("48 83 7B ?? 00 75 ?? B9 11 00 00 00 E8");
    static ref SHARED_SET_WINDOW: Signature = signature!("4C 8B 43 ?? 41 B9 00 00 10 00 BA ?? 00 00 00 48 89 43 ?? 48 8B C8 E8");
    static ref TCP_STATE_SIZE: Signature = signature!("4D 85 ED 75 ?? 48 89 7E ?? E8 ?? ?? ?? ?? 4C 8B F0 E8");
    static ref TCP_TRAIN: Signature = signature!("89 5C ?? ?? 83 FD 01 75 ?? 48 8B 0F E8");
    static ref TCP_DECODE: Signature = signature!("4C 8B 11 48 89 6C ?? ?? 4D 85 D2 74 ?? 49 8B CA E8");
    static ref TCP_ENCODE: Signature = signature!("48 8B ?? 48 8D ?? ?? ?? C6 44 ?? ?? ?? 49 8B ?? 48 89 44 ?? ?? E8");
}

pub struct Noodle {
    // Can't drop the library or the pointers go kaplunk
    #[allow(dead_code)]
    lib: libloading::Library,

    shared_size: SharedSize,
    shared_set_window: SharedSetWindow,
    tcp_state_size: TCPStateSize,
    tcp_train: TCPTrain,
    tcp_decode: TCPDecode,
    tcp_encode: TCPEncode,
}

impl Noodle {
    pub fn new(path: &Path) -> Option<Self> {
        unsafe {
            let lib = libloading::Library::new(path).ok()?;

            let shared_size = SHARED_SIZE.scan_module("ffxiv_dx11.exe").ok()? as *mut u8;
            let shared_set_window =
                SHARED_SET_WINDOW.scan_module("ffxiv_dx11.exe").ok()? as *mut u8;
            let tcp_state_size = TCP_STATE_SIZE.scan_module("ffxiv_dx11.exe").ok()? as *mut u8;
            let tcp_train = TCP_TRAIN.scan_module("ffxiv_dx11.exe").ok()? as *mut u8;
            let tcp_decode = TCP_DECODE.scan_module("ffxiv_dx11.exe").ok()? as *mut u8;
            let tcp_encode = TCP_ENCODE.scan_module("ffxiv_dx11.exe").ok()? as *mut u8;

            Some(Self {
                lib,
                shared_size: std::mem::transmute::<_, SharedSize>(shared_size),
                shared_set_window: std::mem::transmute::<_, SharedSetWindow>(shared_set_window),
                tcp_state_size: std::mem::transmute::<_, TCPStateSize>(tcp_state_size),
                tcp_train: std::mem::transmute::<_, TCPTrain>(tcp_train),
                tcp_decode: std::mem::transmute::<_, TCPDecode>(tcp_decode),
                tcp_encode: std::mem::transmute::<_, TCPEncode>(tcp_encode),
            })
        }
    }

    pub unsafe fn shared_size(&self, bits: i32) -> i32 {
        (self.shared_size)(bits)
    }

    pub unsafe fn shared_set_window(
        &self,
        shared: *mut u8,
        bits: i32,
        window: *mut u8,
        window_size: i32,
    ) {
        (self.shared_set_window)(shared, bits, window, window_size)
    }

    pub unsafe fn tcp_state_size(&self) -> i32 {
        (self.tcp_state_size)()
    }

    pub unsafe fn tcp_train(
        &self,
        state: *mut u8,
        shared: *mut u8,
        training_packet_pointers: *mut u8,
        training_packet_sizes: *mut u8,
        num_training_packets: i32,
    ) {
        (self.tcp_train)(
            state,
            shared,
            training_packet_pointers,
            training_packet_sizes,
            num_training_packets,
        )
    }

    pub unsafe fn tcp_decode(
        &self,
        state: *mut u8,
        shared: *mut u8,
        compressed: *mut u8,
        compressed_size: i32,
        raw: *mut u8,
        raw_size: i32,
    ) -> bool {
        (self.tcp_decode)(state, shared, compressed, compressed_size, raw, raw_size)
    }

    pub unsafe fn tcp_encode(
        &self,
        state: *mut u8,
        shared: *mut u8,
        raw: *mut u8,
        raw_size: i32,
        compressed: *mut u8,
        compressed_size: i32,
    ) -> bool {
        (self.tcp_encode)(state, shared, raw, raw_size, compressed, compressed_size)
    }
}
