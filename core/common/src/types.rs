use sgx_tstd::prelude::v1::*;

pub struct Ciphertext(Vec<u8>);

pub type Address = String;
pub type Value = u64;
pub type Randomness = Vec<u8>;
