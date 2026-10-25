//! Binary entry point: parse the command line and open the window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dupe_finder::cli::{self, Cli};

fn main() -> eframe::Result {
    match cli::parse(std::env::args().skip(1)) {
        Ok(Cli::Help) => {
            println!("{}", cli::USAGE);
            Ok(())
        }
        Ok(Cli::Run(launch)) => dupe_finder::ui::run(launch),
        Err(e) => {
            eprintln!("{e}\n{}", cli::USAGE);
            std::process::exit(2);
        }
    }
}
