//! GF(2^8) finite field arithmetic using AES irreducible polynomial x^8 + x^4 + x^3 + x + 1 (0x11b).

pub const AES_POLYNOMIAL: u16 = 0x11b;

#[inline]
pub fn gf256_add(a: u8, b: u8) -> u8 {
    a ^ b
}

#[inline]
pub fn gf256_sub(a: u8, b: u8) -> u8 {
    a ^ b
}

pub fn gf256_mul(mut a: u8, mut b: u8) -> u8 {
    let mut p: u8 = 0;
    for _ in 0..8 {
        if (b & 1) != 0 {
            p ^= a;
        }
        let hi_bit_set = (a & 0x80) != 0;
        a <<= 1;
        if hi_bit_set {
            a ^= 0x1b; // low 8 bits of 0x11b
        }
        b >>= 1;
    }
    p
}

pub fn gf256_pow(mut base: u8, mut exp: u8) -> u8 {
    let mut res = 1u8;
    while exp > 0 {
        if exp & 1 != 0 {
            res = gf256_mul(res, base);
        }
        base = gf256_mul(base, base);
        exp >>= 1;
    }
    res
}

pub fn gf256_inv(a: u8) -> u8 {
    assert!(a != 0, "GF(2^8) division by zero");
    // By Fermat's Little Theorem in GF(2^8), a^(255 - 1) = a^254 is multiplicative inverse
    gf256_pow(a, 254)
}

pub fn gf256_div(a: u8, b: u8) -> u8 {
    if b == 0 {
        panic!("GF(2^8) division by zero");
    }
    gf256_mul(a, gf256_inv(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gf256_mul_inv() {
        for a in 1..=255 {
            let inv = gf256_inv(a);
            let prod = gf256_mul(a, inv);
            assert_eq!(prod, 1, "Failed inverse for {}", a);
        }
    }
}
