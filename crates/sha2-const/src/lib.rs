//! `const fn` implementation of the SHA-2 family of hash functions.
//!
//! This crate allows you to use the SHA-2 hash functions as constant
//! expressions in Rust. For all other usages, the [`sha2`] crate includes more
//! optimized implementations of these hash functions.
//!
//! [`sha2`]: https://crates.io/crates/sha2
//!
//! # Examples
//!
//! Compute the SHA-256 hash of the Bitcoin genesis block at compile time:
//!
//! ```rust
//! # use sha2_const::Sha256;
//! const VERSION: u32 = 1;
//! const HASH_PREV_BLOCK: [u8; 32] = [0; 32];
//! const HASH_MERKLE_ROOT: [u8; 32] = [
//!     0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f,
//!     0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e,
//!     0x5e, 0x4a,
//! ];
//! const TIME: u32 = 1231006505;
//! const BITS: u32 = 0x1d00ffff;
//! const NONCE: u32 = 0x7c2bac1d;
//!
//! const BLOCK_HASH: [u8; 32] = Sha256::new()
//!     .update(
//!         &Sha256::new()
//!             .update(&VERSION.to_le_bytes())
//!             .update(&HASH_PREV_BLOCK)
//!             .update(&HASH_MERKLE_ROOT)
//!             .update(&TIME.to_le_bytes())
//!             .update(&BITS.to_le_bytes())
//!             .update(&NONCE.to_le_bytes())
//!             .finalize(),
//!     )
//!     .finalize();
//!
//! assert_eq!(
//!     hex::encode(&BLOCK_HASH[..]),
//!     "6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000"
//! );
//! ```
#![no_std]

mod constants;
mod sha;
mod util;

use constants::{H224, H256, H384, H512, H512_224, H512_256};
use util::memcpy;

macro_rules! sha {
    (
        $(#[$doc:meta])* $name:ident,
        $size:literal,
        $inner:ty,
        $iv:ident
    ) => {
        $(#[$doc])*
        #[derive(Clone)]
        pub struct $name {
            inner: $inner,
        }

        impl $name {
            /// The internal block size of the hash function.
            pub const BLOCK_SIZE: usize = <$inner>::BLOCK_SIZE;
            /// The digest size of the hash function.
            pub const DIGEST_SIZE: usize = $size;

            /// Construct a new instance.
            #[expect(clippy::new_without_default)]
            pub const fn new() -> Self {
                Self {
                    inner: <$inner>::new($iv),
                }
            }

            /// Add input data to the hash context.
            #[must_use]
            pub const fn update(mut self, input: &[u8]) -> Self {
                self.inner.update(&input);
                self
            }

            /// Finalize the context and compute the digest.
            #[must_use]
            pub const fn finalize(self) -> [u8; Self::DIGEST_SIZE] {
                let digest = self.inner.finalize();
                let mut truncated = [0; Self::DIGEST_SIZE];
                memcpy(&mut truncated, 0, &digest, 0, Self::DIGEST_SIZE);
                truncated
            }
        }
    };
}

sha!(
    /// The SHA-224 hash function.
    ///
    /// The SHA-256 algorithm with the SHA-224 initialization vector, truncated
    /// to 224 bits.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sha2_const::Sha224;
    /// const DIGEST: [u8; 28] = Sha224::new()
    ///     .update(b"The quick brown fox ")
    ///     .update(b"jumps over the lazy dog")
    ///     .finalize();
    ///
    /// assert_eq!(
    ///     hex::encode(&DIGEST[..]),
    ///     "730e109bd7a8a32b1cb9d9a09aa2325d2430587ddbc0c38bad911525"
    /// );
    /// ```
    Sha224,
    28,
    sha::Sha256,
    H224
);

sha!(
    /// The SHA-256 hash function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sha2_const::Sha256;
    /// const DIGEST: [u8; 32] = Sha256::new()
    ///     .update(b"The quick brown fox ")
    ///     .update(b"jumps over the lazy dog")
    ///     .finalize();
    ///
    /// assert_eq!(
    ///     hex::encode(&DIGEST[..]),
    ///     "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
    /// );
    /// ```
    Sha256,
    32,
    sha::Sha256,
    H256
);

sha!(
    /// The SHA-384 hash function.
    ///
    /// The SHA-512 algorithm with the SHA-384 initialization vector, truncated
    /// to 384 bits.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sha2_const::Sha384;
    /// const DIGEST: [u8; 48] = Sha384::new()
    ///     .update(b"The quick brown fox ")
    ///     .update(b"jumps over the lazy dog")
    ///     .finalize();
    ///
    /// assert_eq!(
    ///     hex::encode(&DIGEST[..]),
    ///     concat!(
    ///         "ca737f1014a48f4c0b6dd43cb177b0afd9e5169367544c49",
    ///         "4011e3317dbf9a509cb1e5dc1e85a941bbee3d7f2afbc9b1"
    ///     )
    /// );
    /// ```
    Sha384,
    48,
    sha::Sha512,
    H384
);

sha!(
    /// The SHA-512 hash function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sha2_const::Sha512;
    /// const DIGEST: [u8; 64] = Sha512::new()
    ///     .update(b"The quick brown fox ")
    ///     .update(b"jumps over the lazy dog")
    ///     .finalize();
    ///
    /// assert_eq!(
    ///     hex::encode(&DIGEST[..]),
    ///     concat!(
    ///         "07e547d9586f6a73f73fbac0435ed76951218fb7d0c8d788a309d785436bbb64",
    ///         "2e93a252a954f23912547d1e8a3b5ed6e1bfd7097821233fa0538f3db854fee6"
    ///     )
    /// );
    /// ```
    Sha512,
    64,
    sha::Sha512,
    H512
);

sha!(
    /// The SHA-512/224 hash function.
    ///
    /// The SHA-512 algorithm with the SHA-512/224 initialization vector,
    /// truncated to 224 bits.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sha2_const::Sha512_224;
    /// const DIGEST: [u8; 28] = Sha512_224::new()
    ///     .update(b"The quick brown fox ")
    ///     .update(b"jumps over the lazy dog")
    ///     .finalize();
    ///
    /// assert_eq!(
    ///     hex::encode(&DIGEST[..]),
    ///     "944cd2847fb54558d4775db0485a50003111c8e5daa63fe722c6aa37"
    /// );
    /// ```
    Sha512_224,
    28,
    sha::Sha512,
    H512_224
);

sha!(
    /// The SHA-512/256 hash function.
    ///
    /// The SHA-512 algorithm with the SHA-512/256 initialization vector,
    /// truncated to 256 bits.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use sha2_const::Sha512_256;
    /// const DIGEST: [u8; 32] = Sha512_256::new()
    ///     .update(b"The quick brown fox ")
    ///     .update(b"jumps over the lazy dog")
    ///     .finalize();
    ///
    /// assert_eq!(
    ///     hex::encode(&DIGEST[..]),
    ///     "dd9d67b371519c339ed8dbd25af90e976a1eeefd4ad3d889005e532fc5bef04d"
    /// );
    /// ```
    Sha512_256,
    32,
    sha::Sha512,
    H512_256
);
