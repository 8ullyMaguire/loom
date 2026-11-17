use loom_store::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub sunset_threshold: u64,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        let sunset_threshold = std::env::var("LOOM_SUNSET_REPORTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);
        Self {
            db,
            sunset_threshold,
        }
    }
}
