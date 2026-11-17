//! A confidence vote is a judgement by one contributor about one tag on one feed.
//! Aggregates are published only at n >= 5; the interval is Wilson 95% over the
//! share of votes with confidence >= 60, rescaled to the 0..100 scale.

pub const MIN_VOTES: usize = 5;
pub const CONFIDENT_THRESHOLD: u8 = 60;
const Z: f64 = 1.96;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vote {
    pub contributor_key: String,
    pub confidence: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Aggregate {
    pub n: usize,
    pub mean: f64,
    pub ci_low: f64,
    pub ci_high: f64,
}

pub fn aggregate(votes: &[Vote]) -> Option<Aggregate> {
    if votes.len() < MIN_VOTES {
        return None;
    }
    let n = votes.len() as f64;
    let mean = votes.iter().map(|v| v.confidence as f64).sum::<f64>() / n;
    let confident = votes
        .iter()
        .filter(|v| v.confidence >= CONFIDENT_THRESHOLD)
        .count() as f64;
    let p = confident / n;
    let denom = 1.0 + Z * Z / n;
    let centre = (p + Z * Z / (2.0 * n)) / denom;
    let half = Z * ((p * (1.0 - p) / n + Z * Z / (4.0 * n * n)).sqrt()) / denom;
    Some(Aggregate {
        n: votes.len(),
        mean,
        ci_low: ((centre - half) * 100.0).max(0.0),
        ci_high: ((centre + half) * 100.0).min(100.0),
    })
}
