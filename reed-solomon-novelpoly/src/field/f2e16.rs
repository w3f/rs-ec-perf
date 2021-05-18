#[cfg(table_bootstrap_complete)]
use super::*;

include!(concat!(env!("OUT_DIR"), "/table_f2e16.rs"));
// impl Additive {
//     pub const ONE: Additive = Additive(???);
// }

// Cantor basis
// pub const BASE: [Elt; FIELD_BITS] =
//    [1_u16, 44234, 15374, 5694, 50562, 60718, 37196, 16402, 27800, 4312, 27250, 47360, 64952, 64308, 65336, 39198];

/// Cantor basis' final element
pub const BASE_FINAL: Elt = 39198;

include!("inc_logarithm.rs");

#[cfg(table_bootstrap_complete)]
include!("inc_afft.rs");

#[cfg(table_bootstrap_complete)]
include!("inc_encode.rs");

#[cfg(table_bootstrap_complete)]
include!("inc_reconstruct.rs");


pub type Elt = u16;
pub type Wide = u32;

/// Quotient ideal generator given by tail of irreducible polynomial
pub const GENERATOR: Elt = 0x2D; // x^16 + x^5 + x^3 + x^2 + 1
const FIELD_BITS = 16;

#[derive(Clone, Copy, Debug, Default, BitXor, BitXorAssign, PartialEq, Eq)]
pub struct GF2e16;

impl FieldAdd for GF2e16 {
    const FIELD_BITS: usize = FIELD_BITS;                                                 
   	const ZERO: Additive = Additive(0);                                                            
    const ONE: Additive = Additive(1);                                                           
}                                                                                                
impl GF2e16 {                                                                                             
         // pub const FIELD_BITS: usize = Additive::FIELD_BITS;                                              
         // pub const FIELD_SIZE: usize = Additive::FIELD_SIZE;                                              
                                                                                                          
//         pub const ONEMASK: Elt = (Additive::FIELD_SIZE - 1) as Elt;                                      //

}


#[test]
fn embedded_gf256() {
    // We've a leaky to_multiplier abstraction that fucks up zero, so start at 1.
    let mask: Elt = !0xFF;
    for i in 1..=255 {
        let i = Additive(i as Elt).to_multiplier();
        for j in 0..256 {
            let j = Additive(j as Elt);
            assert!(j.mul(i).0 & mask == 0);
        }
    }    
}


#[test]
fn flt_roundtrip_small() {
	const N: usize = 16;
	const EXPECTED: [Additive; N] =
		unsafe { std::mem::transmute([1_u16, 2, 3, 5, 8, 13, 21, 44, 65, 0, 0xFFFF, 2, 3, 5, 7, 11]) };

	let mut data = EXPECTED.clone();

	afft(&mut data, N, N / 4);

    /*
	println!("novel basis(rust):");
	data.iter().for_each(|sym| {
		print!(" {:04X}", sym.0);
	});
	println!("");
    */

	inverse_afft(&mut data, N, N / 4);

    assert_eq!(data, EXPECTED);
}
