/* origin: FreeBSD /usr/src/lib/msun/src/e_logf.c */
/**
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
*/
/**
 * Conversion to float by Ian Lance Taylor, Cygnus Support, ian@cygnus.com.
*/

use crate::Float32;

const LN2_HI: Float32 = 6.931_381e-1; /* 0x3f317180 */
const LN2_LO: Float32 = 9.058_001e-6; /* 0x3717f7d1 */
/* |(log(1+s)-log(1-s))/s - Lg(s)| < 2**-34.24 (~[-4.95e-11, 4.97e-11]). */
const LG1: Float32 = 0.666_666_6; /*  0xaaaaaa.0p-24*/
const LG2: Float32 = 0.400_009_72; /*  0xccce13.0p-25 */
const LG3: Float32 = 0.284_987_87; /*  0x91e9ee.0p-25 */
const LG4: Float32 = 0.242_790_79; /*  0xf89e26.0p-26 */

/// Returns the logarithm of `x`
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub fn logf(mut x: Float32) -> Float32 {
    let x1p25 = Float32::from_bits(0x4c000000); // 0x1p25f === 2 ^ 25

    let mut ix = x.to_bits();
    let mut k = 0i32;

    if (ix < 0x00800000) || ((ix >> 31) != 0) {
        /* x < 2**-126  */
        if ix << 1 == 0 {
            return -1. / (x * x); /* log(+-0)=-inf */
        }
        if (ix >> 31) != 0 {
            return (x - x) / 0.; /* log(-#) = NaN */
        }
        /* subnormal number, scale up x */
        k -= 25;
        x *= x1p25;
        ix = x.to_bits();
    } else if ix >= 0x7f800000 {
        return x;
    } else if ix == 0x3f800000 {
        return 0.;
    }

    /* reduce x into [sqrt(2)/2, sqrt(2)] */
    ix += 0x3f800000 - 0x3f3504f3;
    k += ((ix >> 23) as i32) - 0x7f;
    ix = (ix & 0x007fffff) + 0x3f3504f3;
    x = Float32::from_bits(ix);

    let f = x - 1.;
    let s = f / (2. + f);
    let z = s * s;
    let w = z * z;
    let t1 = w * (LG2 + w * LG4);
    let t2 = z * (LG1 + w * LG3);
    let r = t2 + t1;
    let hfsq = 0.5 * f * f;
    let dk = k as Float32;
    s * (hfsq + r) + dk * LN2_LO - hfsq + f + dk * LN2_HI
}