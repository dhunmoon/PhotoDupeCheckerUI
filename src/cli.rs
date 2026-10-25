//! Command-line arguments: `dupe_finder [--scan] [--method NAME] [FOLDER...]`.

use std::path::PathBuf;

use crate::model::Method;

pub const USAGE: &str =
    "Usage: dupe_finder [--scan] [--method content|similar|name|name-size|size] [FOLDER...]";

/// How the GUI should start.
#[derive(Debug, Default, PartialEq)]
pub struct LaunchArgs {
    /// Folders to pre-fill (the first one becomes the Source).
    pub folders: Vec<PathBuf>,
    pub method: Option<Method>,
    /// Start scanning immediately.
    pub auto_scan: bool,
}

#[derive(Debug, PartialEq)]
pub enum Cli {
    Run(LaunchArgs),
    Help,
}

/// Parses arguments (without the program name).
pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Cli, String> {
    let mut launch = LaunchArgs::default();
    let mut args = args.into_iter();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--scan" => launch.auto_scan = true,
            "--method" => {
                let name = args.next().ok_or("--method needs a value")?;
                launch.method =
                    Some(Method::from_cli_name(&name).ok_or(format!("unknown method '{name}'"))?);
            }
            "-h" | "--help" => return Ok(Cli::Help),
            s if s.starts_with("--") => return Err(format!("unknown option '{s}'")),
            _ => launch.folders.push(a.into()),
        }
    }
    Ok(Cli::Run(launch))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(args: &[&str]) -> Result<Cli, String> {
        parse(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn parses_everything() {
        assert_eq!(p(&[]).unwrap(), Cli::Run(LaunchArgs::default()));
        assert_eq!(
            p(&["--scan", "/a", "--method", "similar", "/b"]).unwrap(),
            Cli::Run(LaunchArgs {
                folders: vec!["/a".into(), "/b".into()],
                method: Some(Method::SimilarImage),
                auto_scan: true,
            })
        );
        assert_eq!(p(&["-h"]).unwrap(), Cli::Help);
        assert_eq!(p(&["/a", "--help"]).unwrap(), Cli::Help);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(p(&["--method"]).is_err());
        assert!(p(&["--method", "md5"]).unwrap_err().contains("md5"));
        assert!(p(&["--bogus"]).is_err());
    }
}
