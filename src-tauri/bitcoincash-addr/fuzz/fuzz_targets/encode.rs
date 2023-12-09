#![no_main]
use bitcoincash_addr::Address;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let addr = Address {
        body: data.to_vec(),
        ..Default::default()
    };
    addr.encode();
});
