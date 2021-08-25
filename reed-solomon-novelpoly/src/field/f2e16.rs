#[cfg(table_bootstrap_complete)]
use super::*;
use std::convert::{From, TryInto};
use crate::{FieldAdd, TruncateTo, Logarithm, walsh, Additive};

decl_field_additive!(F2e16, bits = 16, generator = 0x2D, elt = u16, wide = u32, cantor_base_final_elt = 39198);

#[cfg(table_bootstrap_complete)]
use crate::AfftField;
#[cfg(table_bootstrap_complete)]
impl AfftField for F2e16 {}

// #[cfg(table_bootstrap_complete)]
// include!("inc_afft.rs");

// #[test]
// fn embedded_gf256() {
//     // We've a leaky to_multiplier abstraction that fucks up zero, so start at 1.
//     let mask: F2e16::Element = !0xFF;
//     for i in 1..=255 {
//         let i = Additive(i as F2e16::Element).to_multiplier();
//         for j in 0..256 {
//             let j = Additive(j as F2e16::Element);
//             assert!(j.mul(i).0 & mask == 0);
//         }
//     }
// }

// #[test]
// fn flt_roundtrip_small() {
//     const N: usize = 16;
//     const EXPECTED: [Additive<F2e16>; N] =
//         unsafe { std::mem::transmute([1_u16, 2, 3, 5, 8, 13, 21, 44, 65, 0, 0xFFFF, 2, 3, 5, 7, 11]) };

//     let mut data = EXPECTED.clone();

//     AfftField::afft(&mut data, N, N / 4);

//     /*
//     println!("novel basis(rust):");
//     data.iter().for_each(|sym| {
//         print!(" {:04X}", sym.0);
//     });
//     println!("");
//     */
    
//     AfftField::inverse_afft(&mut data, N, N / 4);

//     assert_eq!(data, EXPECTED);
// }
