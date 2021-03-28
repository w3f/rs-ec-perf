#[cfg(table_bootstrap_complete)]
use super::*;

pub type Elt = u8;
pub type Wide = u8;
decl_field_additive!("f16", bits = 4);

pub const ONEMASK: Elt = (FIELD_SIZE - 1) as Elt;

/// Quotient ideal generator given by tail of irreducible polynomial
// Valid Cantor basis, passes embedded_gf16
pub const GENERATOR: Elt = 0x??; //GF(2^8): ??

// /// Cantor basis' final element
// pub const BASE_FINAL: Elt = 230;

include!("inc_logarithm.rs");

// #[cfg(table_bootstrap_complete)]
// include!("inc_afft.rs");

