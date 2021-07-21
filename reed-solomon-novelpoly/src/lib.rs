#![forbid(unused_crate_dependencies)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(destructuring_assignment)]
#![feature(associated_type_defaults)]
#![feature(array_windows)]

pub mod errors;
pub use errors::*;

pub mod util;
pub use util::*;

pub mod field;
pub use self::field::f256;
pub use self::field::f2e16;
pub use self::field::{FieldAdd, FieldMul, TruncateTo, Logarithm, Additive, AfftField, walsh, gf_mul_bitpoly_reduced};

#[macro_use]
pub use self::field::macros;

mod novel_poly_basis;
pub use self::novel_poly_basis::*;

pub mod shard;
pub use self::shard::{Shard};

pub mod wrapped_shard;
pub use self::wrapped_shard::WrappedShard;

#[cfg(feature = "with-alt-cxx-impl")]
pub mod cxx;

#[cfg(test)]
mod test {
    use super::*;
    use reed_solomon_tester::{roundtrip, BYTES, N_SHARDS};

    #[cfg(feature = "naive")]
    #[test]
    fn status_quo_roundtrip() -> Result<()> {
        roundtrip(status_quo::encode::<WrappedShard>, status_quo::reconstruct::<WrappedShard>, &BYTES[..1337], N_SHARDS)
    }

    #[test]
    fn novel_poly_basis_roundtrip() -> Result<()> {
        roundtrip(
            novel_poly_basis::encode::<WrappedShard>,
            novel_poly_basis::reconstruct::<WrappedShard>,
            &BYTES[..1337],
            N_SHARDS,
        )
    }
}
