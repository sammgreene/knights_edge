use sha2::{Digest, Sha256};

pub fn numeric_seed_from_string(input: &str) -> u64 {
    // 1. Hash the input string
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize(); // result is a GenericArray<u8, U32>, essentially 32 bytes

    // 2. Convert the first 8 bytes of the hash into a u64 numeric seed
    // We can use the first 8 bytes as a u64 seed for Pcg32
    // `try_into()` safely converts a slice of length 8 into an [u8; 8]
    let bytes: [u8; 8] = result[0..8].try_into().expect("Slice length is 8");
    u64::from_le_bytes(bytes)
}