extern crate alloc;

use core::{
    fmt,
    default::Default,
    ptr,
    mem,
};
use alloc::{
    boxed::Box,
    vec::Vec,
};
use crate::traits::RawEnclaveTx;

pub const STATE_SIZE: usize = 8;
pub const PUBKEY_SIZE: usize = 32;
pub const ADDRESS_SIZE: usize = 20;
pub const RANDOMNESS_SIZE: usize = 32;
pub const SIG_SIZE: usize = 64;
pub const DB_VALUE_SIZE: usize = STATE_SIZE + RANDOMNESS_SIZE;

pub type Address = [u8; ADDRESS_SIZE];
pub type RawPubkey = [u8; PUBKEY_SIZE];
pub type RawSig = [u8; SIG_SIZE];
pub type RawChallenge = [u8; RANDOMNESS_SIZE];

/// Status for Ecall
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct EnclaveStatus(pub u32);

impl Default for EnclaveStatus {
    fn default() -> Self { EnclaveStatus(0) }
}

impl EnclaveStatus {
    pub fn success() -> Self { EnclaveStatus(0) }

    pub fn error() -> Self { EnclaveStatus(1) }

    pub fn is_err(&self) -> bool {
        match self.0 {
            0 => false,
            _ => true,
        }
    }
}

/// Status for Ocall
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct UntrustedStatus(pub u32);

impl Default for UntrustedStatus {
    fn default() -> Self { UntrustedStatus(0) }
}

impl UntrustedStatus {
    pub fn success() -> Self { UntrustedStatus(0) }

    pub fn error() -> Self { UntrustedStatus(1) }

    pub fn is_err(&self) -> bool {
        match self.0 {
            0 => false,
            _ => true,
        }
    }
}

/// Bridged type from enclave to host to send a JoinGroup transaction.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RawJoinGroupTx {
    /// A pointer to the output of the report using `ocall_save_to_memory()`.
    pub report: *const u8,
    pub report_sig: *const u8,
    pub handshake: *const u8,
}

impl RawEnclaveTx for RawJoinGroupTx {}

impl Default for RawJoinGroupTx {
    fn default() -> Self {
        RawJoinGroupTx {
            report: ptr::null(),
            report_sig: ptr::null(),
            handshake: ptr::null(),
        }
    }
}

impl fmt::Debug for RawJoinGroupTx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_trait_builder = f.debug_struct("RawJoinGroupTx");
        debug_trait_builder.field("report", &(self.report));
        debug_trait_builder.field("report_sig", &(self.report_sig));
        debug_trait_builder.field("handshake", &(self.handshake));
        debug_trait_builder.finish()
    }
}

/// Bridged type from enclave to host to send a handshake transaction.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RawHandshakeTx {
    pub handshake: *const u8,
}

impl RawEnclaveTx for RawHandshakeTx {}

impl Default for RawHandshakeTx {
    fn default() -> Self {
        RawHandshakeTx {
            handshake: ptr::null(),
        }
    }
}

impl fmt::Debug for RawHandshakeTx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_trait_builder = f.debug_struct("RawHandshakeTx");
        debug_trait_builder.field("handshake", &(self.handshake));
        debug_trait_builder.finish()
    }
}

/// Returned from getting state operations.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EnclaveState(pub *const u8);

impl EnclaveState {
    pub fn as_bytes(&self) -> Box<[u8]> {
        let raw_state = self.0 as *mut Box<[u8]>;
        let box_state = unsafe { Box::from_raw(raw_state) };

        *box_state
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.as_bytes().into_vec()
    }
}

impl Default for EnclaveState {
    fn default() -> Self {
        EnclaveState(ptr::null())
    }
}

impl fmt::Debug for EnclaveState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_trait_builder = f.debug_struct("EnclaveState");
        debug_trait_builder.field("0", &(self.0));
        debug_trait_builder.finish()
    }
}

/// Key Value data stored in an Enclave
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct RawUpdatedState {
    pub address: Address,
    pub mem_id: u32,
    pub state: *const u8,
}

impl Default for RawUpdatedState {
    fn default() -> Self {
        RawUpdatedState {
            state: ptr::null(),
            ..unsafe { mem::zeroed() }
        }
    }
}

impl fmt::Debug for RawUpdatedState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_trait_builder = f.debug_struct("RawUpdatedState");
        debug_trait_builder.field("address", &(self.address));
        debug_trait_builder.field("mem_id", &(self.mem_id));
        debug_trait_builder.field("state", &(self.state));
        debug_trait_builder.finish()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum ResultStatus {
    /// Ok = Success = 1.
    Ok = 1,
    /// Failure = Error = 0.
    Failure = 0,
}

impl From<bool> for ResultStatus {
    fn from(i: bool) -> Self {
        if i {
            ResultStatus::Ok
        } else {
            ResultStatus::Failure
        }
    }
}

/// A wrapper to a raw mutable/immutable pointer.
/// The Edger8r will copy the data to the protected stack when you pass a pointer through the EDL.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RawPointer {
    ptr: *const u8,
    _mut: bool,
}

impl RawPointer {
    pub unsafe fn new<T>(_ref: &T) -> Self {
        RawPointer {
            ptr: _ref as *const T as *const u8,
            _mut: false,
        }
    }

    pub unsafe fn new_mut<T>(_ref: &mut T) -> Self {
        RawPointer {
            ptr: _ref as *mut T as *const u8,
            _mut: true,
        }
    }

    pub fn get_ptr<T>(&self) -> *const T {
        self.ptr as *const T
    }

    pub fn get_mut_ptr<T>(&self) -> Result<*mut T, &'static str> {
        if !self._mut {
            Err("This DoublePointer is not mutable")
        } else {
            Ok(self.ptr as *mut T)
        }
    }

    pub unsafe fn get_ref<T>(&self) -> &T {
        &*(self.ptr as *const T)
    }

    pub unsafe fn get_mut_ref<T>(&self) -> Result<&mut T, &'static str> {
        if !self._mut {
            Err("This DoublePointer is not mutable")
        } else {
            Ok(&mut *(self.ptr as *mut T))
        }
    }
}
