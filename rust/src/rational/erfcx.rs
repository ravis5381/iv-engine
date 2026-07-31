//! Cody's scaled complementary error function.
#![allow(clippy::all, clippy::pedantic, clippy::nursery)]

#[inline]
fn dint(x: f64) -> f64 {
    if x > 0.0 {
        x.floor()
    } else {
        -(-x).floor()
    }
}

/// Evaluates `exp(x²) * erfc(x)` using Cody's rational approximations.
#[inline]
pub(crate) fn erfcx_cody(x: f64) -> f64 {
    const A: [f64; 5] = [
        3.1611237438705656,
        113.864154151050156,
        377.485237685302021,
        3209.37758913846947,
        0.185777706184603153,
    ];
    const B: [f64; 4] = [
        23.6012909523441209,
        244.024637934444173,
        1282.61652607737228,
        2844.23683343917062,
    ];
    const C: [f64; 9] = [
        0.564188496988670089,
        8.88314979438837594,
        66.1191906371416295,
        298.635138197400131,
        881.95222124176909,
        1712.04761263407058,
        2051.07837782607147,
        1230.33935479799725,
        2.15311535474403846e-8,
    ];
    const D: [f64; 8] = [
        15.7449261107098347,
        117.693950891312499,
        537.181101862009858,
        1621.38957456669019,
        3290.79923573345963,
        4362.61909014324716,
        3439.36767414372164,
        1230.33935480374942,
    ];
    const P: [f64; 6] = [
        0.305326634961232344,
        0.360344899949804439,
        0.125781726111229246,
        0.0160837851487422766,
        6.58749161529837803e-4,
        0.0163153871373020978,
    ];
    const Q: [f64; 5] = [
        2.56852019228982242,
        1.87295284992346047,
        0.527905102951428412,
        0.0605183413124413191,
        0.00233520497626869185,
    ];
    const XNEG: f64 = -26.628;
    const XHUGE: f64 = 6.71e7;
    const XMAX: f64 = 2.53e307;
    const SQRPI: f64 = 0.56418958354775628695;
    let y = x.abs();
    let mut result;
    if y <= 0.46875 {
        let ysq = if y > 1.11e-16 { y * y } else { 0.0 };
        let mut xnum = A[4] * ysq;
        let mut xden = ysq;
        for i in 0..3 {
            xnum = (xnum + A[i]) * ysq;
            xden = (xden + B[i]) * ysq;
        }
        result = x * (xnum + A[3]) / (xden + B[3]);
        return ysq.exp() * (1.0 - result);
    } else if y <= 4.0 {
        let mut xnum = C[8] * y;
        let mut xden = y;
        for i in 0..7 {
            xnum = (xnum + C[i]) * y;
            xden = (xden + D[i]) * y;
        }
        result = (xnum + C[7]) / (xden + D[7]);
    } else {
        if y >= XMAX {
            result = 0.0;
        } else if y >= XHUGE {
            result = SQRPI / y;
        } else {
            let ysq = 1.0 / (y * y);
            let mut xnum = P[5] * ysq;
            let mut xden = ysq;
            for i in 0..4 {
                xnum = (xnum + P[i]) * ysq;
                xden = (xden + Q[i]) * ysq;
            }
            result = (SQRPI - ysq * (xnum + P[4]) / (xden + Q[4])) / y;
        }
    }
    if x < 0.0 {
        if x < XNEG {
            1.79e308
        } else {
            let ysq = dint(x * 16.0) / 16.0;
            let e = (ysq * ysq).exp() * ((x - ysq) * (x + ysq)).exp();
            result = (e + e) - result;
            result
        }
    } else {
        result
    }
}
