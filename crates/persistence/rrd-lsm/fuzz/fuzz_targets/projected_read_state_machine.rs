#![no_main]

#[path = "../../tests/support/projected_read_model.rs"]
mod projected_read_model;

use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 4_096;

fuzz_target!(|input: &[u8]| {
    if input.len() > MAX_INPUT_BYTES {
        return;
    }
    projected_read_model::run_encoded_scenario(0xf022_5eed_ca81_0001, input);
});
