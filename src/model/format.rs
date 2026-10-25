//! Human-readable formatting helpers.

use std::time::SystemTime;

/// `1536` → `"1.50 KB"`. Uses binary (1024) units.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.2} {}", UNITS[u])
    }
}

/// Local time as `YYYY-MM-DD HH:MM`, or `-` when unknown.
pub fn format_time(t: Option<SystemTime>) -> String {
    match t {
        Some(t) => {
            let dt: chrono::DateTime<chrono::Local> = t.into();
            dt.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "-".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(1023), "1023 B");
        assert_eq!(human_size(1024), "1.00 KB");
        assert_eq!(human_size(1536), "1.50 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.00 MB");
        assert_eq!(human_size(1024u64.pow(5) * 3), "3072.00 TB");
    }

    #[test]
    fn times() {
        assert_eq!(format_time(None), "-");
        let s = format_time(Some(SystemTime::UNIX_EPOCH));
        assert_eq!(s.len(), "1970-01-01 00:00".len());
    }
}
