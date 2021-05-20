#[cfg(table_bootstrap_complete)]
use super::*;

pub type Elt = u8;
pub type Wide = u16;

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

const FIELD_BITS = 16;

/// Cantor basis' final element
pub const BASE_FINAL: Elt = 230;
// pub const BASE_FINAL: Elt = 238;

// /// Cantor basis
// pub const BASE: [Elt; FIELD_BITS] = [1, 214, 152, 146, 86, 200, 88, 230];

#[derive(Clone, Copy, Debug, Default, BitXor, BitXorAssign, PartialEq, Eq)]
pub struct GF2e8(pub Elt);

use crate::logarithm::Logarithm;

#[cfg(table_bootstrap_complete)]
include!("inc_afft.rs");

impl FieldAdd for GF2e16 {
    const FIELD_BITS: usize = FIELD_BITS;                                                 
   	const ZERO: Additive = Additive(0);                                                            
    const ONE: Additive = Additive(1);                                                           
}                                                                                                
impl GF2e16 {                                                                                             
         // pub const FIELD_BITS: usize = Additive::FIELD_BITS;                                              
         // pub const FIELD_SIZE: usize = Additive::FIELD_SIZE;                                              
                                                                                                          
    //         pub const ONEMASK: Elt = (Additive::FIELD_SIZE - 1) as Elt;                                      //
    #[cfg(table_bootstrap_complete)]
    include!(concat!(env!("OUT_DIR"), "/table_f2e16.rs"));


}

