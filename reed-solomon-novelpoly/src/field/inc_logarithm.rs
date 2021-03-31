pub use core::ops::{Mul, MulAssign};

use bit_vec::BitVec;

use super::*;

include!("inc_cantor_basis.rs");

/// Multiplicaiton friendly LOG form of f2e16
#[derive(Clone, Copy, Debug, Add, AddAssign, Sub, SubAssign, PartialEq, Eq)] // Default, PartialOrd,Ord
pub struct Logarithm(pub Elt);

impl Logarithm {
    #[inline(always)]
	pub fn to_wide(self) -> Wide {
		self.0 as Wide
	}
    #[inline(always)]
	pub fn from_wide(x: Wide) -> Logarithm {
		Logarithm(x as Elt)
	}
}

impl Mul<Logarithm> for Logarithm {
    type Output = Additive;

	/// TODO:  Leaky abstraction!  Return a*EXP_TABLE[b] over GF(2^r)
    #[inline(always)]
    #[cfg(table_bootstrap_complete)]
    fn mul(self, other: Logarithm) -> Additive {
		let log = self.0 as Wide + other.0 as Wide;
        // Compute sum of logarithms modulo 2^FIELD_BITS-1 perhaps? 
		let offset = (log & ONEMASK as Wide) + (log >> FIELD_BITS);
		Additive(EXP_TABLE[offset as usize])
    }

    #[cfg(not(table_bootstrap_complete))]
    fn mul(self, other: Logarithm) -> Additive { panic!(); }
}

impl Mul<Logarithm> for Additive {
    type Output = Additive;

	/// Return a*EXP_TABLE[b] over GF(2^r)
    #[inline(always)]
    #[cfg(table_bootstrap_complete)]
    fn mul(self, other: Logarithm) -> Additive {
		if self == Self::ZERO {
			return Self::ZERO;
		}
        self.to_multiplier() * other
    }

    #[cfg(not(table_bootstrap_complete))]
    fn mul(self, other: Logarithm) -> Additive { panic!(); }
}

impl MulAssign<Logarithm> for Additive {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Logarithm) {
        *self = (*self) * rhs;
    }
}

#[cfg(table_bootstrap_complete)]
impl FieldMul<Logarithm> for Additive {
	/// Return multiplier prepared form
    #[inline(always)]
	fn to_multiplier(self) -> Logarithm {
		Logarithm(LOG_TABLE[self.0 as usize])
	}

	/// Multiply field elements by a single multiplier, using SIMD if available
    #[inline(always)]
	fn mul_assign_slice(selfy: &mut [Self], other: Logarithm) {
		for s in selfy {
			*s *= other;
		}
	}
}

#[cfg(table_bootstrap_complete)]
impl MulAssign<Additive> for Additive {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Additive) {
        *self = Additive(EXP_TABLE[((*self).to_multiplier() + rhs.to_multiplier()).0 as usize]);
    }
}



/*
Actually our to_multiplier abstraction is leaky

#[test]
fn multiply_by_zero() {
    let zero_mul = Additive::ZERO.to_multiplier();
    for i in 0..FIELD_SIZE {
        let i = Additive(i as Elt);
        // assert_eq!(Additive::ZERO, Additive::ZERO.mul(i.to_multiplier()) );
        assert_eq!(Additive::ZERO, i.mul(zero_mul) );
    }
}
*/

// We next determine the subfields of E. These are in one-to-one correspon-
// dence with the divisors d of «. Notice that these divisors can all easily be
// found in time « (1). Let d be a divisor of «. Then we can calculate the
// matrix of the F -linear map E —>E that sends each a G E to a a- a, and
// using techniques from linear algebra, we can find a basis for the kernel of this
//     map, which is precisely the unique subfield of E of cardinality p
// [1]H. W. Lenstra Jr, “Finding isomorphisms between finite fields,” Mathematics of Computation, pp. 329–347, 1991.

// why not just find the element of the correct order?
// I'll try that and see if I hit a break wall 

/// compute the norm_GF256/GF16 of to_be_normed
/// element
#[cfg(table_bootstrap_complete)]
fn gf256_get_gf16_generator() -> Additive
{
    //gf256 = <x> then x is of order 255 and x^255 = 1
    //suppose gf16 = <y> then y^15 = 1
    //so 2^4 - 1 | 2^8 - 1 = (2^4 - 1)(2^4 + 1) so we have
    //y = x^(16+1)
    return Additive(EXP_TABLE[LOG_TABLE[((Additive(2).to_multiplier().0 as usize * 17) % FIELD_SIZE) as usize] as usize])
}

/// compute the degree 2 subfield using norm of the generator
/// generate multiplication tables.
#[cfg(table_bootstrap_complete)]
fn compute_gf16_in_g256() ->  [Additive; 16] {
    let gf16_gf256_generator = gf256_get_gf16_generator();
        
    let mut gf16_in_gf256_log_table : [Additive; 16] = [Additive(0); 16] ;
    gf16_in_gf256_log_table[0] = Additive(1);
    for i in 1..16 {
        gf16_in_gf256_log_table[i] = gf16_in_gf256_log_table[i - 1] * gf16_gf256_generator.to_multiplier();
    }
    
    return  gf16_in_gf256_log_table
    
}

/// check if members of a candidate basis is actually linear 
/// independent.
#[cfg(table_bootstrap_complete)]
fn check_linear_independence(candidate_basis: &Vec<Additive>) -> bool {
    let mut basis_matrix: Vec<BitVec<u32>> = Vec::new();
    for i in 0..8 {
        basis_matrix.push(BitVec::from_bytes(&[candidate_basis[i].0 as u8]));
    }    

    linear_algebra_util::determinant(&mut basis_matrix)
}

/// compute the a new basis compatible with the subfield
/// we are bruteforcing on all element and hoping that
/// we find one, though my guess is that always the
/// extension field generator work fine.
#[cfg(table_bootstrap_complete)]
fn find_gf16_compatible_basis() -> Vec<Additive> {
    let y = gf256_get_gf16_generator();
    for i in 2..255 {
        let x = Additive(i);
        let candidate_basis : Vec<Additive> = vec![Additive(1), y, y*y.to_multiplier(), y*y.to_multiplier()*y.to_multiplier(), x, y*x.to_multiplier(), y*y.to_multiplier()*x.to_multiplier(), y*y.to_multiplier()*y.to_multiplier()*x.to_multiplier()];
        if check_linear_independence(&candidate_basis) {
            return candidate_basis
        }            
    }
    panic!("No gf16 compatible basis  was found which seems impossible so I'm panicing")
}

/// gets an elmenet in gf16 compatible basis and transform it to original basis
#[cfg(table_bootstrap_complete)]
fn embed_gf16(gf16_elm: u8, gf16_compatible_basis: &Vec<Additive>) -> Additive {
    let gf16_vec: BitVec = BitVec::from_bytes(&[gf16_elm]);
    let mut embedded_elm: Additive = Additive(0);
    
    for i in 0..8 {
        if gf16_vec[0] {
            embedded_elm.0 = embedded_elm.0 ^ gf16_compatible_basis[i].0;
        }
    }

    embedded_elm
    
}

#[test]
fn embedded_gf16() {
    // We've a leaky to_multiplier abstraction that fucks up zero, so start at 1.
    let gf16_compatible_basis = find_gf16_compatible_basis();    
    let mask: Elt = !0xF;
    for i in 1..16 {
        //let i = Additive(i as Elt).to_multiplier();
        let i = embed_gf16(i, &gf16_compatible_basis).to_multiplier();
        for j in 0..16 { 
            let j = embed_gf16(j, &gf16_compatible_basis).to_multiplier();
            assert!(j.mul(i).0 & mask == 0);
        }
    }
}


/*
#[test]
fn print_gf256() {
    use std::io::Write;
	let mut w = std::fs::OpenOptions::new().create(true).truncate(true).write(true).open(Additive::FIELD_NAME).unwrap();

    write!(w, "\n\n\n{} :\n", Additive::FIELD_NAME);
    for i in 1..=255 {
        write!(w, "{:#b} * .. = ", i);
        let i = Additive(i).to_multiplier();
        for j in 0..=255 {
            let j = Additive(j);
            write!(w, "{:#b} ", j.mul(i).0);
        }
        write!(w, "\n");
    }    
}
*/

