//! Hash calculation and verification utilities.

use bootforge_core::types::{HashResult, VerificationStatus};
use md5::Md5;
use sha2::{Digest, Sha256, Sha512};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Compute SHA256 hash for a file.
pub fn compute_sha256(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute SHA256, SHA512, and MD5 for a given byte slice in memory.
pub fn compute_hashes_from_bytes(data: &[u8]) -> HashResult {
    let mut sha256_hasher = Sha256::new();
    sha256_hasher.update(data);
    let sha256 = format!("{:x}", sha256_hasher.finalize());

    let mut sha512_hasher = Sha512::new();
    sha512_hasher.update(data);
    let sha512 = format!("{:x}", sha512_hasher.finalize());

    let mut md5_hasher = Md5::new();
    md5_hasher.update(data);
    let md5 = format!("{:x}", md5_hasher.finalize());

    HashResult {
        sha256,
        sha512: Some(sha512),
        md5: Some(md5),
        status: VerificationStatus::Unverified,
    }
}

/// Verify a file against an expected SHA256 hash string.
pub fn verify_hash(actual_sha256: &str, expected_sha256: &str) -> VerificationStatus {
    if actual_sha256.trim().eq_ignore_ascii_case(expected_sha256.trim()) {
        VerificationStatus::Verified
    } else {
        VerificationStatus::Mismatch
    }
}
