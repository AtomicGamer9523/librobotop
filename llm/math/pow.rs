/* origin: FreeBSD /usr/src/lib/msun/src/e_pow.c */
/**
 * ====================================================
 * Copyright (C) 2004 by Sun Microsystems, Inc. All rights reserved.
 *
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
*/

// pow(x,y) return x**y
//
//                    n
// Method:  Let x =  2   * (1+f)
//      1. Compute and return log2(x) in two pieces:
//              log2(x) = w1 + w2,
//         where w1 has 53-24 = 29 bit trailing zeros.
//      2. Perform y*log2(x) = n+y' by simulating muti-precision
//         arithmetic, where |y'|<=0.5.
//      3. Return x**y = 2**n*exp(y'*log2)
//
// Special cases:
//      1.  (anything) ** 0  is 1
//      2.  1 ** (anything)  is 1
//      3.  (anything except 1) ** NAN is NAN
//      4.  NAN ** (anything except 0) is NAN
//      5.  +-(|x| > 1) **  +INF is +INF
//      6.  +-(|x| > 1) **  -INF is +0
//      7.  +-(|x| < 1) **  +INF is +0
//      8.  +-(|x| < 1) **  -INF is +INF
//      9.  -1          ** +-INF is 1
//      10. +0 ** (+anything except 0, NAN)               is +0
//      11. -0 ** (+anything except 0, NAN, odd integer)  is +0
//      12. +0 ** (-anything except 0, NAN)               is +INF, raise divbyzero
//      13. -0 ** (-anything except 0, NAN, odd integer)  is +INF, raise divbyzero
//      14. -0 ** (+odd integer) is -0
//      15. -0 ** (-odd integer) is -INF, raise divbyzero
//      16. +INF ** (+anything except 0,NAN) is +INF
//      17. +INF ** (-anything except 0,NAN) is +0
//      18. -INF ** (+odd integer) is -INF
//      19. -INF ** (anything) = -0 ** (-anything), (anything except odd integer)
//      20. (anything) ** 1 is (anything)
//      21. (anything) ** -1 is 1/(anything)
//      22. (-anything) ** (integer) is (-1)**(integer)*(+anything**integer)
//      23. (-anything except 0 and inf) ** (non-integer) is NAN
//
// Accuracy:
//      pow(x,y) returns x**y nearly rounded. In particular
//                      pow(integer,integer)
//      always returns the correct integer provided it is
//      representable.
//
// Constants :
// The hexadecimal values are the intended ones for the following
// constants. The decimal values may be used, provided that the
// compiler will convert from decimal to binary accurately enough
// to produce the hexadecimal values shown.
//

use crate::Float64;

use super::{fabs, get_high_word, scalbn, sqrt, with_set_high_word, with_set_low_word};

const BP: [Float64; 2] = [1.0, 1.5];
const DP_H: [Float64; 2] = [0.0, 5.849_624_872_207_642e-1]; /* 0x3fe2b803_40000000 */
const DP_L: [Float64; 2] = [0.0, 1.350_039_202_129_749e-8]; /* 0x3E4CFDEB, 0x43CFD006 */
const TWO53: Float64 = 9007199254740992.0; /* 0x43400000_00000000 */
const HUGE: Float64 = 1.0e300;
const TINY: Float64 = 1.0e-300;

// poly coefs for (3/2)*(log(x)-2s-2/3*s**3:
const L1: Float64 = 5.999_999_999_999_946e-1; /* 0x3fe33333_33333303 */
const L2: Float64 = 4.285_714_285_785_502e-1; /* 0x3fdb6db6_db6fabff */
const L3: Float64 = 3.333_333_298_183_774_3e-1; /* 0x3fd55555_518f264d */
const L4: Float64 = 2.727_281_238_085_34e-1; /* 0x3fd17460_a91d4101 */
const L5: Float64 = 2.306_607_457_755_617_5e-1; /* 0x3fcd864a_93c9db65 */
const L6: Float64 = 2.069_750_178_003_384_2e-1; /* 0x3fca7e28_4a454eef */
const P1: Float64 = 1.666_666_666_666_660_2e-1; /* 0x3fc55555_5555553e */
const P2: Float64 = -2.777_777_777_701_559_3e-3; /* 0xbf66c16c_16bebd93 */
const P3: Float64 = 6.613_756_321_437_934e-5; /* 0x3f11566a_af25de2c */
const P4: Float64 = -1.653_390_220_546_525_2e-6; /* 0xbebbbd41_c5d26bf1 */
const P5: Float64 = 4.138_136_797_057_238_5e-8; /* 0x3e663769_72bea4d0 */
const LG2: Float64 = 6.931_471_805_599_453e-1; /* 0x3fe62e42_fefa39ef */
const LG2_H: Float64 = 6.931_471_824_645_996e-1; /* 0x3fe62e43_00000000 */
const LG2_L: Float64 = -1.904_654_299_957_768e-9; /* 0xbe205c61_0ca86c39 */
const OVT: Float64 = 8.008_566_259_537_294e-17; /* -(1024-log2(ovfl+.5ulp)) */
const CP: Float64 = 9.617_966_939_259_756e-1; /* 0x3feec709_dc3a03fd =2/(3ln2) */
const CP_H: Float64 = 9.617_967_009_544_373e-1; /* 0x3feec709_e0000000 =(float)cp */
const CP_L: Float64 = -7.028_461_650_952_758e-9; /* 0xbe3e2fe0_145b01f5 =tail of cp_h*/
const IVLN2: Float64 = 1.442_695_040_888_963_4; /* 0x3ff71547_652b82fe =1/ln2 */
const IVLN2_H: Float64 = 1.442_695_021_629_333_5; /* 0x3ff71547_60000000 =24b 1/ln2*/
const IVLN2_L: Float64 = 1.925_962_991_126_617_5e-8; /* 0x3e54ae0b_f85ddf44 =1/ln2 tail*/

/// Returns `x` raised to the power `y`.
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub fn pow(x: Float64, y: Float64) -> Float64 {
    let t1: Float64;
    let t2: Float64;

    let (hx, lx): (i32, u32) = ((x.to_bits() >> 32) as i32, x.to_bits() as u32);
    let (hy, ly): (i32, u32) = ((y.to_bits() >> 32) as i32, y.to_bits() as u32);

    let mut ix: i32 = hx & 0x7fffffff;
    let iy: i32 = hy & 0x7fffffff;

    /* x**0 = 1, even if x is NaN */
    if ((iy as u32) | ly) == 0 {
        return 1.0;
    }

    /* 1**y = 1, even if y is NaN */
    if hx == 0x3ff00000 && lx == 0 {
        return 1.0;
    }

    /* NaN if either arg is NaN */
    if ix > 0x7ff00000
        || (ix == 0x7ff00000 && lx != 0)
        || iy > 0x7ff00000
        || (iy == 0x7ff00000 && ly != 0)
    {
        return x + y;
    }

    /* determine if y is an odd int when x < 0
     * yisint = 0       ... y is not an integer
     * yisint = 1       ... y is an odd int
     * yisint = 2       ... y is an even int
     */
    let mut yisint: i32 = 0;
    let mut k: i32;
    let mut j: i32;
    if hx < 0 {
        if iy >= 0x43400000 {
            yisint = 2; /* even integer y */
        } else if iy >= 0x3ff00000 {
            k = (iy >> 20) - 0x3ff; /* exponent */

            if k > 20 {
                j = (ly >> (52 - k)) as i32;

                if (j << (52 - k)) == (ly as i32) {
                    yisint = 2 - (j & 1);
                }
            } else if ly == 0 {
                j = iy >> (20 - k);

                if (j << (20 - k)) == iy {
                    yisint = 2 - (j & 1);
                }
            }
        }
    }

    if ly == 0 {
        /* special value of y */
        if iy == 0x7ff00000 {
            /* y is +-inf */

            return if ((ix - 0x3ff00000) | (lx as i32)) == 0 {
                /* (-1)**+-inf is 1 */
                1.0
            } else if ix >= 0x3ff00000 {
                /* (|x|>1)**+-inf = inf,0 */
                if hy >= 0 {
                    y
                } else {
                    0.0
                }
            } else {
                /* (|x|<1)**+-inf = 0,inf */
                if hy >= 0 {
                    0.0
                } else {
                    -y
                }
            };
        }

        if iy == 0x3ff00000 {
            /* y is +-1 */
            return if hy >= 0 { x } else { 1.0 / x };
        }

        if hy == 0x40000000 {
            /* y is 2 */
            return x * x;
        }

        if hy == 0x3fe00000 {
            /* y is 0.5 */
            if hx >= 0 {
                /* x >= +0 */
                return sqrt(x);
            }
        }
    }

    let mut ax: Float64 = fabs(x);
    if lx == 0 {
        /* special value of x */
        if ix == 0x7ff00000 || ix == 0 || ix == 0x3ff00000 {
            /* x is +-0,+-inf,+-1 */
            let mut z: Float64 = ax;

            if hy < 0 {
                /* z = (1/|x|) */
                z = 1.0 / z;
            }

            if hx < 0 {
                if ((ix - 0x3ff00000) | yisint) == 0 {
                    z = (z - z) / (z - z); /* (-1)**non-int is NaN */
                } else if yisint == 1 {
                    z = -z; /* (x<0)**odd = -(|x|**odd) */
                }
            }

            return z;
        }
    }

    let mut s: Float64 = 1.0; /* sign of result */
    if hx < 0 {
        if yisint == 0 {
            /* (x<0)**(non-int) is NaN */
            return (x - x) / (x - x);
        }

        if yisint == 1 {
            /* (x<0)**(odd int) */
            s = -1.0;
        }
    }

    /* |y| is HUGE */
    if iy > 0x41e00000 {
        /* if |y| > 2**31 */
        if iy > 0x43f00000 {
            /* if |y| > 2**64, must o/uflow */
            if ix <= 0x3fefffff {
                return if hy < 0 { HUGE * HUGE } else { TINY * TINY };
            }

            if ix >= 0x3ff00000 {
                return if hy > 0 { HUGE * HUGE } else { TINY * TINY };
            }
        }

        /* over/underflow if x is not close to one */
        if ix < 0x3fefffff {
            return if hy < 0 {
                s * HUGE * HUGE
            } else {
                s * TINY * TINY
            };
        }
        if ix > 0x3ff00000 {
            return if hy > 0 {
                s * HUGE * HUGE
            } else {
                s * TINY * TINY
            };
        }

        /* now |1-x| is TINY <= 2**-20, suffice to compute
        log(x) by x-x^2/2+x^3/3-x^4/4 */
        let t: Float64 = ax - 1.0; /* t has 20 trailing zeros */
        let w: Float64 = (t * t) * (0.5 - t * (0.333_333_333_333_333_3 - t * 0.25));
        let u: Float64 = IVLN2_H * t; /* ivln2_h has 21 sig. bits */
        let v: Float64 = t * IVLN2_L - w * IVLN2;
        t1 = with_set_low_word(u + v, 0);
        t2 = v - (t1 - u);
    } else {
        // double ss,s2,s_h,s_l,t_h,t_l;
        let mut n: i32 = 0;

        if ix < 0x00100000 {
            /* take care subnormal number */
            ax *= TWO53;
            n -= 53;
            ix = get_high_word(ax) as i32;
        }

        n += (ix >> 20) - 0x3ff;
        j = ix & 0x000fffff;

        /* determine interval */
        let k: i32;
        ix = j | 0x3ff00000; /* normalize ix */
        if j <= 0x3988E {
            /* |x|<sqrt(3/2) */
            k = 0;
        } else if j < 0xBB67A {
            /* |x|<sqrt(3)   */
            k = 1;
        } else {
            k = 0;
            n += 1;
            ix -= 0x00100000;
        }
        ax = with_set_high_word(ax, ix as u32);

        /* compute ss = s_h+s_l = (x-1)/(x+1) or (x-1.5)/(x+1.5) */
        let u: Float64 = ax - i!(BP, k as usize); /* bp[0]=1.0, bp[1]=1.5 */
        let v: Float64 = 1.0 / (ax + i!(BP, k as usize));
        let ss: Float64 = u * v;
        let s_h = with_set_low_word(ss, 0);

        /* t_h=ax+bp[k] High */
        let t_h: Float64 = with_set_high_word(
            0.0,
            ((ix as u32 >> 1) | 0x20000000) + 0x00080000 + ((k as u32) << 18),
        );
        let t_l: Float64 = ax - (t_h - i!(BP, k as usize));
        let s_l: Float64 = v * ((u - s_h * t_h) - s_h * t_l);

        /* compute log(ax) */
        let s2: Float64 = ss * ss;
        let mut r: Float64 = s2 * s2 * (L1 + s2 * (L2 + s2 * (L3 + s2 * (L4 + s2 * (L5 + s2 * L6)))));
        r += s_l * (s_h + ss);
        let s2: Float64 = s_h * s_h;
        let t_h: Float64 = with_set_low_word(3.0 + s2 + r, 0);
        let t_l: Float64 = r - ((t_h - 3.0) - s2);

        /* u+v = ss*(1+...) */
        let u: Float64 = s_h * t_h;
        let v: Float64 = s_l * t_h + t_l * ss;

        /* 2/(3log2)*(ss+...) */
        let p_h: Float64 = with_set_low_word(u + v, 0);
        let p_l = v - (p_h - u);
        let z_h: Float64 = CP_H * p_h; /* cp_h+cp_l = 2/(3*log2) */
        let z_l: Float64 = CP_L * p_h + p_l * CP + i!(DP_L, k as usize);

        /* log2(ax) = (ss+..)*2/(3*log2) = n + dp_h + z_h + z_l */
        let t: Float64 = n as Float64;
        t1 = with_set_low_word(((z_h + z_l) + i!(DP_H, k as usize)) + t, 0);
        t2 = z_l - (((t1 - t) - i!(DP_H, k as usize)) - z_h);
    }

    /* split up y into y1+y2 and compute (y1+y2)*(t1+t2) */
    let y1: Float64 = with_set_low_word(y, 0);
    let p_l: Float64 = (y - y1) * t1 + y * t2;
    let mut p_h: Float64 = y1 * t1;
    let z: Float64 = p_l + p_h;
    let mut j: i32 = (z.to_bits() >> 32) as i32;
    let i: i32 = z.to_bits() as i32;
    // let (j, i): (i32, i32) = ((z.to_bits() >> 32) as i32, z.to_bits() as i32);

    if j >= 0x40900000 {
        /* z >= 1024 */
        if (j - 0x40900000) | i != 0 {
            /* if z > 1024 */
            return s * HUGE * HUGE; /* overflow */
        }

        if p_l + OVT > z - p_h {
            return s * HUGE * HUGE; /* overflow */
        }
    } else if (j & 0x7fffffff) >= 0x4090cc00 {
        /* z <= -1075 */
        // FIXME: instead of abs(j) use unsigned j

        if (((j as u32) - 0xc090cc00) | (i as u32)) != 0 {
            /* z < -1075 */
            return s * TINY * TINY; /* underflow */
        }

        if p_l <= z - p_h {
            return s * TINY * TINY; /* underflow */
        }
    }

    /* compute 2**(p_h+p_l) */
    let i: i32 = j & 0x7fffffff_i32;
    k = (i >> 20) - 0x3ff;
    let mut n: i32 = 0;

    if i > 0x3fe00000 {
        /* if |z| > 0.5, set n = [z+0.5] */
        n = j + (0x00100000 >> (k + 1));
        k = ((n & 0x7fffffff) >> 20) - 0x3ff; /* new k for n */
        let t: Float64 = with_set_high_word(0.0, (n & !(0x000fffff >> k)) as u32);
        n = ((n & 0x000fffff) | 0x00100000) >> (20 - k);
        if j < 0 {
            n = -n;
        }
        p_h -= t;
    }

    let t: Float64 = with_set_low_word(p_l + p_h, 0);
    let u: Float64 = t * LG2_H;
    let v: Float64 = (p_l - (t - p_h)) * LG2 + t * LG2_L;
    let mut z: Float64 = u + v;
    let w: Float64 = v - (z - u);
    let t: Float64 = z * z;
    let t1: Float64 = z - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    let r: Float64 = (z * t1) / (t1 - 2.0) - (w + z * w);
    z = 1.0 - (r - z);
    j = get_high_word(z) as i32;
    j += n << 20;

    if (j >> 20) <= 0 {
        /* subnormal output */
        z = scalbn(z, n);
    } else {
        z = with_set_high_word(z, j as u32);
    }

    s * z
}

#[cfg(test)]
mod tests {
    extern crate core;
    use super::Float64;

    use self::core::f64::consts::{E, PI};
    use self::core::f64::{EPSILON, INFINITY, MAX, MIN, MIN_POSITIVE, NAN, NEG_INFINITY};
    use super::pow;

    const POS_ZERO: &[Float64] = &[0.0];
    const NEG_ZERO: &[Float64] = &[-0.0];
    const POS_ONE: &[Float64] = &[1.0];
    const NEG_ONE: &[Float64] = &[-1.0];
    const POS_FLOATS: &[Float64] = &[99.0 / 70.0, E, PI];
    const NEG_FLOATS: &[Float64] = &[-99.0 / 70.0, -E, -PI];
    const POS_SMALL_FLOATS: &[Float64] = &[(1.0 / 2.0), MIN_POSITIVE, EPSILON];
    const NEG_SMALL_FLOATS: &[Float64] = &[-(1.0 / 2.0), -MIN_POSITIVE, -EPSILON];
    const POS_EVENS: &[Float64] = &[2.0, 6.0, 8.0, 10.0, 22.0, 100.0, MAX];
    const NEG_EVENS: &[Float64] = &[MIN, -100.0, -22.0, -10.0, -8.0, -6.0, -2.0];
    const POS_ODDS: &[Float64] = &[3.0, 7.0];
    const NEG_ODDS: &[Float64] = &[-7.0, -3.0];
    const NANS: &[Float64] = &[NAN];
    const POS_INF: &[Float64] = &[INFINITY];
    const NEG_INF: &[Float64] = &[NEG_INFINITY];

    const ALL: &[&[Float64]] = &[
        POS_ZERO,
        NEG_ZERO,
        NANS,
        NEG_SMALL_FLOATS,
        POS_SMALL_FLOATS,
        NEG_FLOATS,
        POS_FLOATS,
        NEG_EVENS,
        POS_EVENS,
        NEG_ODDS,
        POS_ODDS,
        NEG_INF,
        POS_INF,
        NEG_ONE,
        POS_ONE,
    ];
    const POS: &[&[Float64]] = &[POS_ZERO, POS_ODDS, POS_ONE, POS_FLOATS, POS_EVENS, POS_INF];
    const NEG: &[&[Float64]] = &[NEG_ZERO, NEG_ODDS, NEG_ONE, NEG_FLOATS, NEG_EVENS, NEG_INF];

    fn pow_test(base: Float64, exponent: Float64, expected: Float64) {
        let res = pow(base, exponent);
        assert!(
            if expected.is_nan() {
                res.is_nan()
            } else {
                pow(base, exponent) == expected
            },
            "{} ** {} was {} instead of {}",
            base,
            exponent,
            res,
            expected
        );
    }

    fn test_sets_as_base(sets: &[&[Float64]], exponent: Float64, expected: Float64) {
        sets.iter()
            .for_each(|s| s.iter().for_each(|val| pow_test(*val, exponent, expected)));
    }

    fn test_sets_as_exponent(base: Float64, sets: &[&[Float64]], expected: Float64) {
        sets.iter()
            .for_each(|s| s.iter().for_each(|val| pow_test(base, *val, expected)));
    }

    fn test_sets(sets: &[&[Float64]], computed: &dyn Fn(Float64) -> Float64, expected: &dyn Fn(Float64) -> Float64) {
        sets.iter().for_each(|s| {
            s.iter().for_each(|val| {
                let exp = expected(*val);
                let res = computed(*val);

                #[cfg(all(target_arch = "x86", not(target_feature = "sse2")))]
                let exp = force_eval!(exp);
                #[cfg(all(target_arch = "x86", not(target_feature = "sse2")))]
                let res = force_eval!(res);
                assert!(
                    if exp.is_nan() {
                        res.is_nan()
                    } else {
                        exp == res
                    },
                    "test for {} was {} instead of {}",
                    val,
                    res,
                    exp
                );
            })
        });
    }

    #[test]
    fn zero_as_exponent() {
        test_sets_as_base(ALL, 0.0, 1.0);
        test_sets_as_base(ALL, -0.0, 1.0);
    }

    #[test]
    fn one_as_base() {
        test_sets_as_exponent(1.0, ALL, 1.0);
    }

    #[test]
    fn nan_inputs() {
        // NAN as the base:
        // (NAN ^ anything *but 0* should be NAN)
        test_sets_as_exponent(NAN, &ALL[2..], NAN);

        // NAN as the exponent:
        // (anything *but 1* ^ NAN should be NAN)
        test_sets_as_base(&ALL[..(ALL.len() - 2)], NAN, NAN);
    }

    #[test]
    fn infinity_as_base() {
        // Positive Infinity as the base:
        // (+Infinity ^ positive anything but 0 and NAN should be +Infinity)
        test_sets_as_exponent(INFINITY, &POS[1..], INFINITY);

        // (+Infinity ^ negative anything except 0 and NAN should be 0.0)
        test_sets_as_exponent(INFINITY, &NEG[1..], 0.0);

        // Negative Infinity as the base:
        // (-Infinity ^ positive odd ints should be -Infinity)
        test_sets_as_exponent(NEG_INFINITY, &[POS_ODDS], NEG_INFINITY);

        // (-Infinity ^ anything but odd ints should be == -0 ^ (-anything))
        // We can lump in pos/neg odd ints here because they don't seem to
        // cause panics (div by zero) in release mode (I think).
        test_sets(ALL, &|v: Float64| pow(NEG_INFINITY, v), &|v: Float64| pow(-0.0, -v));
    }

    #[test]
    fn infinity_as_exponent() {
        // Positive/Negative base greater than 1:
        // (pos/neg > 1 ^ Infinity should be Infinity - note this excludes NAN as the base)
        test_sets_as_base(&ALL[5..(ALL.len() - 2)], INFINITY, INFINITY);

        // (pos/neg > 1 ^ -Infinity should be 0.0)
        test_sets_as_base(&ALL[5..ALL.len() - 2], NEG_INFINITY, 0.0);

        // Positive/Negative base less than 1:
        let base_below_one = &[POS_ZERO, NEG_ZERO, NEG_SMALL_FLOATS, POS_SMALL_FLOATS];

        // (pos/neg < 1 ^ Infinity should be 0.0 - this also excludes NAN as the base)
        test_sets_as_base(base_below_one, INFINITY, 0.0);

        // (pos/neg < 1 ^ -Infinity should be Infinity)
        test_sets_as_base(base_below_one, NEG_INFINITY, INFINITY);

        // Positive/Negative 1 as the base:
        // (pos/neg 1 ^ Infinity should be 1)
        test_sets_as_base(&[NEG_ONE, POS_ONE], INFINITY, 1.0);

        // (pos/neg 1 ^ -Infinity should be 1)
        test_sets_as_base(&[NEG_ONE, POS_ONE], NEG_INFINITY, 1.0);
    }

    #[test]
    fn zero_as_base() {
        // Positive Zero as the base:
        // (+0 ^ anything positive but 0 and NAN should be +0)
        test_sets_as_exponent(0.0, &POS[1..], 0.0);

        // (+0 ^ anything negative but 0 and NAN should be Infinity)
        // (this should panic because we're dividing by zero)
        test_sets_as_exponent(0.0, &NEG[1..], INFINITY);

        // Negative Zero as the base:
        // (-0 ^ anything positive but 0, NAN, and odd ints should be +0)
        test_sets_as_exponent(-0.0, &POS[3..], 0.0);

        // (-0 ^ anything negative but 0, NAN, and odd ints should be Infinity)
        // (should panic because of divide by zero)
        test_sets_as_exponent(-0.0, &NEG[3..], INFINITY);

        // (-0 ^ positive odd ints should be -0)
        test_sets_as_exponent(-0.0, &[POS_ODDS], -0.0);

        // (-0 ^ negative odd ints should be -Infinity)
        // (should panic because of divide by zero)
        test_sets_as_exponent(-0.0, &[NEG_ODDS], NEG_INFINITY);
    }

    #[test]
    fn special_cases() {
        // One as the exponent:
        // (anything ^ 1 should be anything - i.e. the base)
        test_sets(ALL, &|v: Float64| pow(v, 1.0), &|v: Float64| v);

        // Negative One as the exponent:
        // (anything ^ -1 should be 1/anything)
        test_sets(ALL, &|v: Float64| pow(v, -1.0), &|v: Float64| 1.0 / v);

        // Factoring -1 out:
        // (negative anything ^ integer should be (-1 ^ integer) * (positive anything ^ integer))
        [POS_ZERO, NEG_ZERO, POS_ONE, NEG_ONE, POS_EVENS, NEG_EVENS]
            .iter()
            .for_each(|int_set| {
                int_set.iter().for_each(|int| {
                    test_sets(ALL, &|v: Float64| pow(-v, *int), &|v: Float64| {
                        pow(-1.0, *int) * pow(v, *int)
                    });
                })
            });

        // Negative base (imaginary results):
        // (-anything except 0 and Infinity ^ non-integer should be NAN)
        NEG[1..(NEG.len() - 1)].iter().for_each(|set| {
            set.iter().for_each(|val| {
                test_sets(&ALL[3..7], &|v: Float64| pow(*val, v), &|_| NAN);
            })
        });
    }

    #[test]
    fn normal_cases() {
        assert_eq!(pow(2.0, 20.0), (1 << 20) as Float64);
        assert_eq!(pow(-1.0, 9.0), -1.0);
        assert!(pow(-1.0, 2.2).is_nan());
        assert!(pow(-1.0, -1.14).is_nan());
    }
}