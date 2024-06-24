const HQ_LOOKUP: [u8; 101] = [
    1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8,
    9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 14, 14, 14, 15, 15, 15, 16, 16, 17,
    17, 17, 18, 18, 18, 19, 19, 20, 20, 21, 22, 23, 24, 26, 28, 31, 34, 38, 42, 47, 52, 58, 64, 68,
    71, 74, 76, 78, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 94, 96, 98, 100,
];

pub fn hq_percentage(quality: u32, max_quality: u32) -> u8 {
    // TODO: switch to std::num::NonZeroU32 at some point
    assert!(max_quality != 0, "max_quality must be non-zero");
    let ratio = quality as f64 / max_quality as f64;
    HQ_LOOKUP[(ratio * 100.0).floor() as usize]
}
