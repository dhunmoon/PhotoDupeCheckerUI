//! # Dupe Finder
//!
//! A desktop app for finding and cleaning up duplicate files, with special support for
//! photos saved in different formats / sizes.
//!
//! ## Architecture
//!
//! | Module          | Responsibility                                                  |
//! |-----------------|-----------------------------------------------------------------|
//! | [`model`]       | Plain data: config, files, groups, file kinds, formatting       |
//! | [`scan`]        | Background scan pipeline (walk → group → sort)                  |
//! | [`hashing`]     | Fast exact hashing (size → partial → full xxh3)                 |
//! | [`phash`]       | Perceptual image hashing, fast JPEG decode, BK-tree             |
//! | [`selection`]   | "Smart select" rules deciding which copies to mark               |
//! | [`actions`]     | Trash / delete / move / copy / link the marked files safely     |
//! | [`report`]      | CSV export                                                      |
//! | [`cli`]         | Command-line parsing                                            |
//! | [`ui`]          | egui front end (no business logic beyond view state)            |
//!
//! Everything except [`ui`] is GUI-independent and unit tested.

pub mod actions;
pub mod cli;
pub mod hashing;
pub mod model;
pub mod phash;
pub mod report;
pub mod scan;
pub mod selection;
pub mod ui;

#[cfg(test)]
pub(crate) mod test_util;
