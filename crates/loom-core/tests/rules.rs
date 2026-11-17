use confidence::Vote;
use loom_core::{bundle, confidence, gravity, sunset};

#[test]
fn bundle_requires_one_to_one_hundred_feeds_and_keeps_order() {
    assert!(bundle::Bundle::new("empty", vec![]).is_err());
    let uris: Vec<String> = (0..100)
        .map(|i| format!("at://did/app.bsky.feed.generator/f{i}"))
        .collect();
    let b = bundle::Bundle::new("hundred", uris.clone()).expect("100 feeds accepted");
    assert_eq!(b.uris, uris);
    assert_eq!(b.edit_token.len(), 43, "32 random bytes in base64");
    let mut too_many = uris;
    too_many.push("at://one/too/many".into());
    assert!(bundle::Bundle::new("too many", too_many).is_err());
}

#[test]
fn confidence_needs_five_independent_votes() {
    let four: Vec<Vote> = (0..4)
        .map(|i| Vote {
            contributor_key: format!("k{i}"),
            confidence: 80,
        })
        .collect();
    assert!(confidence::aggregate(&four).is_none());
    let mut five = four;
    five.push(Vote {
        contributor_key: "k4".into(),
        confidence: 60,
    });
    let a = confidence::aggregate(&five).expect("five votes publish");
    assert_eq!(a.n, 5);
    assert!((a.mean - 76.0).abs() < 1e-6);
    assert!(a.ci_low <= a.mean && a.mean <= a.ci_high);
    assert!(a.ci_low >= 0.0 && a.ci_high <= 100.0);
}

#[test]
fn sunset_is_a_declared_count() {
    assert!(!sunset::should_close(19, 20));
    assert!(sunset::should_close(20, 20));
    assert!(sunset::should_close(21, 20));
}

#[test]
fn gravity_ramp_is_zero_at_arrival_monotone_and_capped() {
    assert_eq!(gravity::ramp(0, 0.25, 10.0), 0.0);
    let a = gravity::ramp(1, 0.25, 10.0);
    let b = gravity::ramp(60, 0.25, 10.0);
    let c = gravity::ramp(100_000, 0.25, 10.0);
    assert!(a > 0.0 && b > a && c > b);
    assert!(c < 0.25);
    assert!(gravity::score(1.0, 0.9, b) > gravity::score(1.0, 0.1, b));
}
