//! ADR 0001 — every game number is a custom big-float: `mantissa * 10^exponent`.
//!
//! Normalized so the value is either exactly zero (`mantissa == 0.0`) or has
//! `1.0 <= |mantissa| < 10.0`. This is the only numeric type the sim speaks;
//! raw `f64`/`u128` arithmetic never touches game values.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Beyond this exponent gap, adding the smaller number is a no-op: an `f64`
/// mantissa holds ~15-16 significant digits, so anything 17 orders of magnitude
/// smaller vanishes under the larger's precision.
const NEGLIGIBLE_GAP: i64 = 17;

/// Suffixes for short display, each step = 1e3. Past the end we fall back to
/// scientific notation.
const SUFFIXES: [&str; 11] = [
    "", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "No",
];

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Big {
    #[serde(rename = "m")]
    pub mantissa: f64,
    #[serde(rename = "e")]
    pub exponent: i64,
}

#[inline]
fn sign_of(m: f64) -> i32 {
    if m > 0.0 {
        1
    } else if m < 0.0 {
        -1
    } else {
        0
    }
}

impl Big {
    pub const ZERO: Big = Big { mantissa: 0.0, exponent: 0 };
    pub const ONE: Big = Big { mantissa: 1.0, exponent: 0 };

    /// Construct and normalize.
    pub fn new(mantissa: f64, exponent: i64) -> Big {
        let mut b = Big { mantissa, exponent };
        b.normalize();
        b
    }

    pub fn from_f64(v: f64) -> Big {
        Big::new(v, 0)
    }

    fn normalize(&mut self) {
        if !self.mantissa.is_finite() {
            // Defensive: a non-finite value should never arise in normal play.
            // Collapse to zero rather than poison every later comparison.
            *self = Big::ZERO;
            return;
        }
        if self.mantissa == 0.0 {
            self.exponent = 0;
            return;
        }
        let sign = if self.mantissa < 0.0 { -1.0 } else { 1.0 };
        let mut m = self.mantissa.abs();
        let mut e = self.exponent;
        while m >= 10.0 {
            m /= 10.0;
            e += 1;
        }
        while m < 1.0 {
            m *= 10.0;
            e -= 1;
        }
        self.mantissa = sign * m;
        self.exponent = e;
    }

    pub fn is_zero(&self) -> bool {
        self.mantissa == 0.0
    }

    pub fn add(self, other: Big) -> Big {
        if self.is_zero() {
            return other;
        }
        if other.is_zero() {
            return self;
        }
        // Align to the larger exponent.
        let (big, small) = if self.exponent >= other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        let diff = big.exponent - small.exponent;
        if diff > NEGLIGIBLE_GAP {
            return big;
        }
        // Bring `small`'s mantissa down into `big`'s exponent scale and combine.
        let combined = big.mantissa + small.mantissa * 10f64.powi(-(diff as i32));
        Big::new(combined, big.exponent)
    }

    pub fn neg(self) -> Big {
        Big { mantissa: -self.mantissa, exponent: self.exponent }
    }

    pub fn sub(self, other: Big) -> Big {
        self.add(other.neg())
    }

    pub fn mul(self, other: Big) -> Big {
        if self.is_zero() || other.is_zero() {
            return Big::ZERO;
        }
        Big::new(self.mantissa * other.mantissa, self.exponent + other.exponent)
    }

    pub fn div(self, other: Big) -> Big {
        if other.is_zero() || self.is_zero() {
            // Division by zero is defended rather than panicking; no game path
            // should hit it, but a corrupt save must not crash the loop.
            return Big::ZERO;
        }
        Big::new(self.mantissa / other.mantissa, self.exponent - other.exponent)
    }

    pub fn mul_f64(self, k: f64) -> Big {
        self.mul(Big::from_f64(k))
    }

    /// Total order over big-floats. Sign first, then magnitude (exponent, then
    /// mantissa). The inherent method so `Ord::cmp` can delegate without
    /// recursing.
    pub fn compare(&self, other: &Big) -> Ordering {
        let ss = sign_of(self.mantissa);
        let os = sign_of(other.mantissa);
        if ss != os {
            return ss.cmp(&os);
        }
        if ss == 0 {
            return Ordering::Equal; // both zero
        }
        let magnitude = if self.exponent != other.exponent {
            self.exponent.cmp(&other.exponent)
        } else {
            self.mantissa
                .abs()
                .partial_cmp(&other.mantissa.abs())
                .unwrap_or(Ordering::Equal)
        };
        if ss < 0 {
            magnitude.reverse()
        } else {
            magnitude
        }
    }

    /// `self >= other` — the common affordability / summit check.
    pub fn gte(&self, other: &Big) -> bool {
        self.compare(other) != Ordering::Less
    }

    /// log10 of the magnitude, for log-scaled UI (progress bars). Undefined for
    /// zero; callers guard with `is_zero`.
    pub fn log10(&self) -> f64 {
        self.exponent as f64 + self.mantissa.abs().log10()
    }

    /// Collapse to an `f64` for small values (counts, ratios). Saturates to
    /// infinity past `f64` range — only call on values known to be small.
    pub fn to_f64(&self) -> f64 {
        self.mantissa * 10f64.powi(self.exponent as i32)
    }

    /// Human-readable form: plain for small, suffixed up to 1e33, scientific
    /// beyond.
    pub fn format(&self) -> String {
        if self.is_zero() {
            return "0".to_string();
        }
        let neg = self.mantissa < 0.0;
        let e = self.exponent;
        let body = if e < 3 {
            let v = self.mantissa.abs() * 10f64.powi(e as i32);
            format!("{}", v.floor() as i64)
        } else if (e as usize) < SUFFIXES.len() * 3 {
            let group = (e / 3) as usize;
            let lead = self.mantissa.abs() * 10f64.powi((e % 3) as i32);
            format!("{:.2}{}", lead, SUFFIXES[group])
        } else {
            format!("{:.2}e{}", self.mantissa.abs(), e)
        };
        if neg {
            format!("-{}", body)
        } else {
            body
        }
    }
}

impl Default for Big {
    fn default() -> Big {
        Big::ZERO
    }
}

impl PartialEq for Big {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other) == Ordering::Equal
    }
}
impl Eq for Big {}
impl PartialOrd for Big {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare(other))
    }
}
impl Ord for Big {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_into_range() {
        let b = Big::new(12345.0, 0);
        assert!((b.mantissa - 1.2345).abs() < 1e-12);
        assert_eq!(b.exponent, 4);
    }

    #[test]
    fn zero_is_canonical() {
        let b = Big::new(0.0, 99);
        assert!(b.is_zero());
        assert_eq!(b.exponent, 0);
        assert_eq!(Big::ZERO, Big::new(0.0, 5));
    }

    #[test]
    fn add_same_scale() {
        let r = Big::from_f64(100.0).add(Big::from_f64(50.0));
        assert_eq!(r, Big::from_f64(150.0));
    }

    #[test]
    fn add_negligible_small_is_noop() {
        let big = Big::new(1.0, 100);
        let small = Big::from_f64(1.0);
        // 1e100 + 1 == 1e100 at f64 precision.
        assert_eq!(big.add(small), big);
    }

    #[test]
    fn subtraction_works() {
        let r = Big::from_f64(150.0).sub(Big::from_f64(150.0));
        assert!(r.is_zero());
        let r2 = Big::from_f64(200.0).sub(Big::from_f64(75.0));
        assert_eq!(r2, Big::from_f64(125.0));
    }

    #[test]
    fn multiplication_adds_exponents() {
        let r = Big::new(2.0, 50).mul(Big::new(3.0, 50));
        assert_eq!(r, Big::new(6.0, 100));
    }

    #[test]
    fn division_subtracts_exponents() {
        let r = Big::new(6.0, 100).div(Big::new(3.0, 40));
        assert_eq!(r, Big::new(2.0, 60));
    }

    #[test]
    fn ordering_by_magnitude_then_sign() {
        assert!(Big::new(1.0, 100) > Big::new(9.0, 99));
        assert!(Big::from_f64(-5.0) < Big::ZERO);
        assert!(Big::ZERO < Big::from_f64(0.001));
        assert!(Big::new(5.0, 10) > Big::new(4.9, 10));
    }

    #[test]
    fn formats_readably() {
        assert_eq!(Big::from_f64(42.0).format(), "42");
        assert_eq!(Big::new(1.5, 6).format(), "1.50M");
        assert_eq!(Big::new(2.0, 9).format(), "2.00B");
        assert_eq!(Big::new(1.0, 100).format(), "1.00e100");
        assert_eq!(Big::ZERO.format(), "0");
    }

    #[test]
    fn survives_idle_scale() {
        // The whole point of ADR 0001: numbers a u128 could never hold.
        let huge = Big::new(1.0, 1000);
        let doubled = huge.add(huge);
        assert!(doubled > huge);
        assert_eq!(doubled, Big::new(2.0, 1000));
    }

    #[test]
    fn serde_round_trip() {
        let b = Big::new(3.14159, 42);
        let s = serde_json::to_string(&b).unwrap();
        let back: Big = serde_json::from_str(&s).unwrap();
        assert_eq!(b, back);
    }
}
