#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

mod cxx {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use crate::Shard;

fn setup() {
    use std::sync::Once;

    static SETUP: Once = Once::new();

    SETUP.call_once(|| unsafe {
        cxx::setup();
    });
}

pub fn encode<S: Shard>(_bytes: &[u8]) -> Vec<S> {
    setup();
    unimplemented!("encode for C for usage in rs bench is not implemented")
}

pub fn reconstruct<S: Shard>(_received_shards: Vec<Option<S>>) -> Option<Vec<u8>> {
    setup();
    unimplemented!("reconstruction for C for usage in rs bench is not implemented")
}
