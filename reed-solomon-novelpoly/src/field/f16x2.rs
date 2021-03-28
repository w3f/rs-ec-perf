#[cfg(table_bootstrap_complete)]
use super::*;

pub type Elt = u8;
pub type Wide = u16;
decl_field_additive!("f256", bits = 8);

/// Quotient ideal generator given by tail of irreducible polynomial
// Valid Cantor basis, passes embedded_gf16
pub const GENERATOR: Elt = 0x1D; //GF(2^8): x^8 + x^4 + x^3 + x^2 + 1
// pub const GENERATOR: Elt = 0x71; //GF(2^8): z^8 + z^6 + z^5 + z^4 + 1
// pub const GENERATOR: Elt = 0x2B; //GF(2^8): x^8 + x^5 + x^3 + x + 1
// pub const GENERATOR: Elt = 0x2D; //GF(2^8): x^8 + x^5 + x^3 + x^2 + 1

// Valid Cantor basis, but fails embedded_gf16

// Valid Cantor basis, but fails both embedded_gf16 and b_is_one.
// pub const GENERATOR: Elt = 0x1B; //GF(2^8): x^8 + x^4 + x^3 + x + 1
// pub const GENERATOR: Elt = 0x3F; //GF(2^8): x^8 + x^5 + x^4 + x^3 + x^2 + x + 1
// pub const GENERATOR: Elt = 0x39; //GF(2^8): x^8 + x^5 + x^4 + x^3 + 1
// pub const GENERATOR: Elt = 0x77; //GF(2^8): z^8 + z^6 + z^5 + z^4 + z^3 + 1

// Is this Chen's suggested tower?  Does not yield a Cantor basis.
// pub const GENERATOR: Elt = 0x7B; //GF(2^8): z^8 + z^6 + z^5 + z^4 + z^3 + z + 1

// Select for GFNI compatability, but lacks an embedded GF(16).
// pub const GENERATOR: Elt = 0x1B; //GF(2^8): x^8 + x^4 + x^3 + x + 1


// impl Additive {
//     pub const ONE: Additive = Additive(???);
// }

/// Cantor basis' final element
pub const BASE_FINAL: Elt = 230;
// pub const BASE_FINAL: Elt = 238;

// /// Cantor basis
// pub const BASE: [Elt; FIELD_BITS] = [1, 214, 152, 146, 86, 200, 88, 230];

include!("inc_logarithm.rs");


pub use super::f16 as half;

impl Additive {
    #[inline(always)]
    pub fn split_x2(self) -> [half::Additive; 2] {
        [half::Additive( self.0 & 0xF ), half::Additive( (self.0 & 0xF0) >> 4 )]
    }
}

impl half::Logarithm {
    const MUL_X2_R0: Logarithm = Logarithm( unimplmented!() );
    const MUL_X2_R1: Logarithm = Logarithm( unimplmented!() );
}

impl half::Additive {
    // #[inline(always)]
    // pub fn mul_x2_r0(x: half::Additive) -> half::Additive {
    //     unimplmented!();
    // }

    #[inline(always)]
    pub fn mul_x2_r1(x: half::Additive) -> half::Additive {
        unimplmented!();
    }
}

pub fn join_x2(x: [half::Additive; 2]) -> Additive {
    debug_assert!(x[0] & 0xF == 0);
    debug_assert!(x[1] & 0xF == 0);    
    Additive( x[0] | (x[1] << 4) )
}

include!("inc_log_x2.rs");


#[cfg(table_bootstrap_complete)]
include!("inc_afft.rs");

