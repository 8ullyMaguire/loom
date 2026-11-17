//! The sunset is a declared numeric rule, never a manual toggle.

pub fn should_close(reports_30d: u64, threshold: u64) -> bool {
    reports_30d >= threshold
}
