//! Exporting scan results.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::model::{DupGroup, format_time};

/// CSV header written by [`to_csv`].
pub const CSV_HEADER: &str = "group,path,size_bytes,modified,width,height,role,distance,marked";

/// Renders all groups as CSV (one row per file). `marked` is the current selection.
pub fn to_csv(groups: &[DupGroup], marked: &HashSet<PathBuf>) -> String {
    let mut out = String::from(CSV_HEADER);
    out.push('\n');
    for g in groups {
        for f in &g.files {
            let (w, h) = f
                .dims
                .map(|(w, h)| (w.to_string(), h.to_string()))
                .unwrap_or_default();
            out.push_str(&format!(
                "{},{},{},{},{},{},{},{},{}\n",
                g.id,
                csv_escape(&f.path.to_string_lossy()),
                f.size,
                format_time(f.modified),
                w,
                h,
                f.role.label(),
                f.distance.map(|d| d.to_string()).unwrap_or_default(),
                marked.contains(&f.path)
            ));
        }
    }
    out
}

/// Always quotes, doubling any embedded quotes (RFC 4180).
fn csv_escape(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FileEntry;

    #[test]
    fn csv_rows_and_escaping() {
        let mut a = FileEntry::new("/p/he said \"hi\", ok.jpg", 10);
        a.dims = Some((4, 3));
        a.distance = Some(2);
        let b = FileEntry::new("/p/b.jpg", 10);
        let groups = vec![DupGroup {
            id: 7,
            label: String::new(),
            files: vec![a, b],
        }];
        let marked: HashSet<PathBuf> = [PathBuf::from("/p/b.jpg")].into();
        let csv = to_csv(&groups, &marked);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], CSV_HEADER);
        assert_eq!(
            lines[1],
            "7,\"/p/he said \"\"hi\"\", ok.jpg\",10,-,4,3,Source,2,false"
        );
        assert_eq!(lines[2], "7,\"/p/b.jpg\",10,-,,,Source,,true");
    }
}
