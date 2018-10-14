//! handshake math.

use bincode::serialize;
use crypto::hmac::Hmac;
use crypto::mac::{Mac, MacResult};
use crypto::sha3::Sha3;
use failure::Error;
use rand::{OsRng, RngCore};
use serde_derive::{Deserialize, Serialize};

/// Key is a crytographic key used to authenticate state cookies.
#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    bytes: [u8; Key::BYTES],
}

impl Key {
    const BYTES: usize = 32;

    /// Returns a new key using random bytes from the OS Rng.
    pub fn new() -> Result<Self, Error> {
        let mut rng = OsRng::new()?;
        let mut bytes = [0; Key::BYTES];
        rng.fill_bytes(&mut bytes);
        Ok(Key { bytes })
    }
}

/// State cookies are used in the four way connection handshake. Usage is based on SCTP; look there
/// for further information.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateCookie {
    pub tcb:  Tcb,
    pub hmac: [u8; Key::BYTES],
}

impl StateCookie {
    /// Creates a new state cookie signed by the given key.
    pub fn new(tcb: Tcb, key: &Key) -> Self {
        let hmac = tcb.hmac(key);
        Self { tcb, hmac }
    }

    /// Returns true if the state cookie was signed using the given key. Uses invariable time
    /// comparison.
    pub fn signed_by(&self, key: &Key) -> bool {
        MacResult::new(&self.hmac) == MacResult::new(&self.tcb.hmac(key))
    }
}

/// Tcb contains all the information needed to manage an established connection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tcb {
    pub our_tsn:     u32,
    pub our_token:   u32,
    pub their_tsn:   u32,
    pub their_token: u32,
}

impl Tcb {
    /// Returns an HMAC for the tcb content using the key.
    fn hmac(&self, key: &Key) -> [u8; Key::BYTES] {
        let mut hmac_gen = Hmac::new(Sha3::sha3_256(), &key.bytes);
        hmac_gen.input(&serialize(self).unwrap());

        let mut hmac = [0; Key::BYTES];
        hmac.copy_from_slice(&hmac_gen.result().code());

        hmac
    }
}
