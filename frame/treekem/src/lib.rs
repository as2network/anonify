#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

mod application;
mod crypto;
mod group_state;
pub mod handshake;
mod ratchet_tree;
mod tree_math;
// #[cfg(debug_assertions)]
mod test_funcs;

pub use crate::application::AppKeyChain;
pub use crate::crypto::dh::{DhPrivateKey, DhPubKey};
pub use crate::crypto::ecies::EciesCiphertext;
pub use crate::crypto::secrets::{PathSecret, SealedPathSecret, UnsealedPathSecret};
pub use crate::group_state::GroupState;
pub use crate::handshake::Handshake;
pub use crate::test_funcs::init_path_secret_kvs;

#[cfg(debug_assertions)]
pub mod tests {
    use super::*;
    use std::prelude::v1::*;
    use test_utils::*;

    pub fn run_tests() -> bool {
        check_all_passed!(
            application::tests::run_tests(),
            crypto::ecies::tests::run_tests(),
            crypto::secrets::tests::run_tests(),
        )
    }
}
