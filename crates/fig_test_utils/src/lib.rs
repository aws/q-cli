pub mod server;

use hex::ToHex;
pub use http;
pub use server::*;

/// Encodes the byte slice as a lower-case sha256 hex string.
pub fn sha256(data: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, data).encode_hex()
}
