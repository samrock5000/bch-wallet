#![no_main]
use bitcoincash_addr::Address;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    Address::decode(&hex::encode(data));
});
