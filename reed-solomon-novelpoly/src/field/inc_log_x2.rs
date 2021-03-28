

/// Prepared multiplicaiton double LOG form for degree two extension field
///
/// T^2 = r1 T + r0 implies
///
/// (x T + y) (u T + v)
/// = x u T^2 + (y u + x v) T + y v
/// = (y u + x (r1 u + v)) T + (y v + x r0 u)
///
/// u=0 implies
/// (x T + y) v = x v T + y v
/// 
/// v=0 implies
/// (x T + y) u T
/// = x u T^2 + y u T
/// = (y u + x r1 u) T + x r0 u
///
/// x=0 implies
/// y (u T + v) = y u T + y v
///
/// y=0 implies
/// x T (u T + v)
/// = x u T^2 + x v T
/// = x (r1 u + v) T + x r0 u
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogX2 {
    Zero,
    NvZu {
        v: half::Logarithm,
    },
    ZvNu {
        u: half::Logarithm
        r0_u: half::Logarithm,
        r1_u: half::Logarithm,
    },
    NvNu {
        u: half::Logarithm,
        v: half::Logarithm,
        r0_u: half::Logarithm,
        r1_u_plus_v: half::Logarithm,
    }
}

/*
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogX2 {
    v: Option<half::Logarithm>,
    u: Option<LogX2u>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogX2u {
    u: half::Logarithm,
    r0_u: half::Logarithm,
    r1_u_plus_v: half::Logarithm,
}
*/

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogX2u {
    u: half::Logarithm,
    r0_u: half::Logarithm,
    r1_u_plus_v: half::Logarithm,
}

impl Mul<LogX2> for Additive {
    type Output = Additive;

    #[inline(always)]
    #[cfg(table_bootstrap_complete)]
    fn mul(self, other: LogX2) -> Additive {
		if self == Self::ZERO || other == LogX2::Zero { return Self::ZERO; }
        // if self == Self::ZERO || (other.v == None && other.v == None) { return Self::ZERO; }
        let [y_a,x_a] = self.split_x2();
        let mut (yv,yu) = if y_a != half::Additive::ZERO {
            let y = y_a.to_multiplier();
            let f = |o| o.(|w| y * w).unwrap_or(half::Additive::ZERO)
            // (
            //    other.v.and_then(|v| y * v).unwrap_or(half::Additive::ZERO),
            //    other.v.and_then(|u| y * u.u).unwrap_or(half::Additive::ZERO),
            // )
            match other {
                LogX2::Zero => panic!(),
                LogX2::NvZu { v } => ( y * v, half::Additive::ZERO ),
                LogX2::ZvNu { u, .. } => ( half::Additive::ZERO, y * u ),
                LogX2::NvNu { u, v, .. } => ( y * v, y * u ),
            }
        } else { (half::Additive::ZERO, half::Additive::ZERO) };
        if x_a != half::Additive::ZERO {
            let x = x_a.to_multiplier();
            // if let Some(u) = other.u {
            //    yv ^= x * u.r0_u;
            //    yu ^= x * u.r1_u_plus_v;
            // }
            match other {
                LogX2::Zero => panic!(),
                LogX2::NvZu { _v } => { },
                LogX2::ZvNu { _u, r0_u, r1_u, } => {
                    yv ^= x * r0_u;
                    yu ^= x * r1_u;
                },
                LogX2::NvNu { _u, _v, r0_u, r1_u_plus_v, } => {
                    yv ^= x * r0_u;
                    yu ^= x * r1_u_plus_v;
                },
            }
        }
        join_x2([yv, xv])
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
        if self == Additive::ZERO { return LogX2::Zero; }
        let [v_a,u_a] = self.split_x2();
        let v = v_a.to_multiplier();
        if u_a == half::Additive::ZERO {
            return LogX2::NvZu { v, };
        }
        let u = u_a.to_multiplier();
        let r0_u = u + MUL_X2_R0; // Faster using addition
        // debug_assert_eq!( r1_u, u.mul_x2_r0().to_multiplier() );
        if v_a == half::Additive::ZERO {
            let r1_u = u + MUL_X2_R1; // Faster using addition
            debug_assert_eq!( r1_u, u.mul_x2_r1().to_multiplier() );
            LogX2::ZvNu { u, r0_u, r1_u, }
        } else {
            let r1_u_plus_v = (u.mul_x2_r1() ^ v).to_multiplier();
            LogX2::NvNu { u, v, r0_u, r1_u_plus_v, }
        }
	}

	/// Multiply field elements by a single multiplier, using SIMD if available
    #[inline(always)]
	fn mul_assign_slice(selfy: &mut [Self], other: Logarithm) {
		for s in selfy {
			*s *= other;
		}
	}
}


#[test]
fn embedded_gf16() {
    // We've a leaky to_multiplier abstraction that fucks up zero, so start at 1.
    let mask: Elt = !0xF;
    for i in 1..16 {
        let i = Additive(i as Elt).to_multiplier();
        for j in 0..16 {
            let j = Additive(j as Elt);
            assert!(j.mul(i).0 & mask == 0);
        }
    }
}

