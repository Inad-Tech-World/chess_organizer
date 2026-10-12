//! FIDE Rating Regulations (B.02) conversion tables.

/// Table 8.1(a): rating difference `dp` for a fractional score `p`, for
/// p = 0.50, 0.51 … 1.00 (index = hundredths above 0.50).
const DP: [i32; 51] = [
    0, 7, 14, 21, 29, 36, 43, 50, 57, 65, 72, 80, 87, 95, 102, 110, 117, 125, 133, 141, 149, 158,
    166, 175, 184, 193, 202, 211, 220, 230, 240, 251, 262, 273, 284, 296, 309, 322, 336, 351, 366,
    383, 401, 422, 444, 470, 501, 538, 589, 677, 800,
];

/// `dp` for a score fraction, rounded to two decimals as FIDE does.
pub fn dp(p: f64) -> i32 {
    let hundredths = (p.clamp(0.0, 1.0) * 100.0).round() as i32;
    if hundredths >= 50 {
        DP[(hundredths - 50) as usize]
    } else {
        -DP[(50 - hundredths) as usize]
    }
}

/// Table 8.1(b): upper bounds of rating-difference bands; band `i` gives
/// the higher-rated player an expected score of 0.50 + i/100.
const PD_UPPER: [i32; 50] = [
    3, 10, 17, 25, 32, 39, 46, 53, 61, 68, 76, 83, 91, 98, 106, 113, 121, 129, 137, 145, 153, 162,
    170, 179, 188, 197, 206, 215, 225, 235, 245, 256, 267, 278, 290, 302, 315, 328, 344, 357, 374,
    391, 411, 432, 456, 484, 517, 559, 619, 735,
];

/// Expected score (in hundredths) of a player whose rating exceeds the
/// opponent's by `diff` (which may be negative). No 400-point cap.
pub fn expected_hundredths(diff: i32) -> i32 {
    let d = diff.abs();
    let higher = PD_UPPER
        .iter()
        .position(|&ub| d <= ub)
        .map_or(100, |i| 50 + i as i32);
    if diff >= 0 { higher } else { 100 - higher }
}

/// Expected score with the 400-point rule (B.02 8.3.1) applied.
pub fn expected_capped(diff: i32) -> f64 {
    f64::from(expected_hundredths(diff.clamp(-400, 400))) / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dp_is_symmetric() {
        assert_eq!(dp(1.0), 800);
        assert_eq!(dp(0.0), -800);
        assert_eq!(dp(0.5), 0);
        assert_eq!(dp(0.75), 193);
        assert_eq!(dp(0.25), -193);
        assert_eq!(dp(0.666), 125);
    }

    #[test]
    fn expected_scores() {
        assert_eq!(expected_hundredths(0), 50);
        assert_eq!(expected_hundredths(3), 50);
        assert_eq!(expected_hundredths(4), 51);
        assert_eq!(expected_hundredths(-4), 49);
        assert_eq!(expected_hundredths(735), 99);
        assert_eq!(expected_hundredths(736), 100);
        assert!((expected_capped(800) - 0.92).abs() < 1e-9);
    }
}
