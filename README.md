# Dupe Finder

A fast desktop app (Rust + egui) for finding and cleaning up duplicate files. It also
finds **the same photo saved in a different format, size or quality**.

## Features

- **Five ways to match files:**
  - **Identical content** (recommended). Files are checked in stages: size first, then a
    quick hash of the start, middle and end, then a full xxh3 hash. Only likely matches
    get read in full. A *fast mode* skips the full hash.
  - **Similar-looking images.** A 128-bit perceptual hash (dHash) finds resized,
    re-compressed or converted copies (JPG ↔ PNG ↔ WebP …). It can also match rotated or
    mirrored copies, and a slider sets how strict the match is.
  - **Same file name** (case and extension can be ignored), **Same name + size**,
    **Same size**.
- **Two compare modes:** *Everything together*, or *Source → Target*. The second mode
  only reports files in Target folders (for example a backup) that already exist in a
  Source folder (your originals).
- **Recursive scanning** with filters for file type, size, hidden files, symlinks and
  path patterns to skip.
- **Three result views:** a list, a thumbnail grid, and a side-by-side view where ★
  marks the best value of each property.
- **Smart select** chooses one copy to keep in every group, based on: highest
  resolution, largest/smallest, newest/oldest, shortest/longest path, shortest name,
  Source folders, or a folder you pick. You can also mark files by extension or by text
  in the path.
- **Actions on marked files:** move to Trash, move or copy to a folder (optionally
  keeping the folder structure), replace with hard links or symlinks, or delete
  permanently. Folders left empty can be removed afterwards.
- **Safety:**
  - Before acting on a file, the app checks it again and skips it if it changed since
    the scan.
  - Groups where every copy is marked are skipped by default.
  - A confirmation dialog appears before anything is changed.
- Export the results to CSV, and see unreadable files and action results in the log.

## Download & install

Get the latest build from the
[Releases page](https://github.com/dhunmoon/PhotoDupeCheckerUI/releases/latest):

| System | File |
|---|---|
| Windows 10/11 (64-bit) | `…-windows-x86_64-setup.exe` (installer) or `…-windows-x86_64.zip` (portable) |
| macOS, Apple Silicon | `…-macos-apple-silicon.dmg` |
| macOS, Intel | `…-macos-intel.dmg` |
| Ubuntu / Debian / Mint | `…_amd64.deb` — `sudo apt install ./dupe-finder_*.deb` |
| Fedora / RHEL / openSUSE | `…x86_64.rpm` — `sudo dnf install ./dupe-finder-*.rpm` |
| Any other Linux | `…-linux-x86_64.AppImage` (`chmod +x`, then run) or the `.tar.gz` |

The Linux builds need glibc 2.28 or newer: Debian 10+, Ubuntu 18.10+, RHEL/Rocky 8+,
Fedora 29+, or any current Arch/openSUSE.

The builds are not code-signed yet, so the first launch shows a warning:

- **Windows:** in *"Windows protected your PC"*, click **More info → Run anyway**.
- **macOS:** drag the app to Applications and open it. If macOS refuses, open **System
  Settings → Privacy & Security** and click **Open Anyway**. You can also run
  `xattr -dr com.apple.quarantine "/Applications/Dupe Finder.app"`.

## Running from source

```sh
cargo run --release
# optional: pre-fill folders, choose a method and start scanning immediately
cargo run --release -- --scan --method similar ~/Pictures ~/Backup/Pictures
```

`--method` accepts `content` (default), `similar`, `name`, `name-size` and `size`.
The first folder becomes the **Source** and later folders become **Targets**. In the
app, click a folder's tag to change its role.

## Using the app

1. **Choose folders.** Click ➕ Add, or drag folders onto the window.
2. **Compare.** Pick *Everything together*, or *Source → Target* to clean a backup
   against your originals.
3. **Find duplicates by.** Pick a method. Its options appear underneath it.
4. Press **Scan for duplicates**.
5. Review the results. Each file shows **✔ KEEP** or **✖ REMOVE**. Click a file to
   switch it, or use **✨ Smart select** to mark every group at once. Right-click a file
   for *Keep only this one*, *Open* and *Show in folder*.
6. Click **🗑 Move N to Trash**, or open **More actions** for move, copy, link, delete
   and export.

## Why not just hash the first N pixels or the border?

Decoding the image is the slow part, not hashing it. Reading "the first N pixels" of a
JPEG/PNG still means decoding the file up to that point. Those pixels also change
completely when the image is resized or re-encoded, so they can't match a resized copy.
Borders are often plain (sky, white margins) and are the first thing cropping removes,
so they cause both false matches and misses.

This app uses two cheaper techniques instead:

- **Exact duplicates:** it samples *bytes* (64 KB from the start, middle and end, plus
  the size) without decoding anything. See [`src/hashing.rs`](src/hashing.rs).
- **Similar images:** it decodes JPEGs at 1/2, 1/4 or 1/8 scale, which is several times
  faster, and hashes a 64×64 grayscale version. A BK-tree avoids comparing every pair
  of images. See [`src/phash/mod.rs`](src/phash/mod.rs).

## Project layout

```text
src/
  main.rs            entry point (parse CLI, open window)
  lib.rs             crate docs + module list
  cli.rs             command-line parsing
  model/             plain data: config, files/groups, file kinds, formatting
  scan/              background pipeline: collect → content | similar | name/size → sort
  hashing.rs         size → partial → full xxh3 hashing
  phash/             fast image loading, perceptual hash, BK-tree
  selection.rs       smart-select rules
  actions/           trash/delete/move/copy/link + safe file-system helpers
  report.rs          CSV export
  ui/
    mod.rs           DupeApp + frame layout
    theme.rs         design system: palette, spacing, reusable widgets
    fonts.rs         bundled Inter + Phosphor font setup
    icons.rs         the icon set (Phosphor codepoints)
    logo.rs          app logo (window icon + header)
    results.rs       result/filter/mark state (no egui, unit tested)
    jobs.rs          background scan/action threads
    sidebar.rs       setup steps + scan button
    top_bar.rs       title + headline numbers
    action_bar.rs    smart select + actions
    dialogs.rs       confirm dialog + log window
    welcome.rs       guide / progress / "no duplicates" screens
    thumbs.rs        background thumbnail cache
    views/           list, grid, side-by-side + shared drawing helpers
tests/end_to_end.rs  scan → select → act flows through the public API
```

To browse the API docs, run `cargo doc --no-deps --open`.

## Development

```sh
cargo test                     # 85 unit + end-to-end tests
cargo clippy --all-targets
cargo fmt
```

## Releasing

Releases are built by GitHub Actions ([`.github/workflows/release.yml`](.github/workflows/release.yml)).
Every push and pull request also runs fmt, clippy and the tests on Linux, Windows and
macOS ([`ci.yml`](.github/workflows/ci.yml)).

1. Bump `version` in `Cargo.toml`, then run `cargo build` so `Cargo.lock` is updated.
   Commit both files.
2. Tag the commit and push the tag:
   ```sh
   git tag v0.1.0
   git push origin main v0.1.0
   ```
3. The workflow checks that the tag matches `Cargo.toml`, builds every platform, and
   publishes a GitHub Release. The release includes the files below, a `SHA256SUMS.txt`,
   the install notes from [`packaging/release-notes.md`](packaging/release-notes.md),
   and auto-generated change notes. Tags with a hyphen, such as `v0.2.0-beta.1`, become
   pre-releases.

To test the builds without publishing, open the **Actions** tab and choose **Release →
Run workflow**. The files are attached to that run.

| Platform | Built on | Output |
|---|---|---|
| Linux x86_64 | ubuntu-latest, `cargo zigbuild` against glibc 2.28 | `.tar.gz`, `.AppImage`, `.deb`, `.rpm` |
| macOS Apple Silicon / Intel | macos-latest (Intel is cross-compiled), min macOS 11 | `.dmg` with `Dupe Finder.app`, ad-hoc signed |
| Windows x86_64 | windows-latest (MSVC), icon embedded by `build.rs` | Inno Setup installer + portable `.zip` |

The packaging scripts in [`packaging/`](packaging) also run locally. For example,
`packaging/package-linux.sh` needs `cargo-zigbuild` with zig, `cargo-deb` and
`cargo-generate-rpm`. The deb/rpm metadata is in `Cargo.toml` under
`[package.metadata.*]`. The platform icons (`packaging/windows/app.ico`,
`packaging/macos/AppIcon.icns`, `packaging/linux/dupe-finder.png`) are generated from
`assets/logo/app-icon.png`. Regenerate them if the logo changes.

**Code signing (later):** a signed build removes the first-launch warnings. macOS needs
an Apple Developer ID ($99/year) plus notarization. Windows needs a code-signing
certificate, or Azure Trusted Signing. Both can be added to `release.yml` as extra steps
that use repository secrets.

## Design & assets

The UI uses a small design system in [`src/ui/theme.rs`](src/ui/theme.rs): one palette per
light/dark mode, a 4/8/12/16/24 px spacing scale, three control heights (26/32/40 px),
and shared widgets (buttons, segmented control, badges, chips, menu items). Build new UI
from these helpers so everything stays consistent.

The dark palette is a "Blue & Slate" token set: app background `#0B0F17`, sidebar/bars
`#0F172A`, surfaces `#1E293B` (hover `#243247`), 1 px borders `#2D3B4F` (focus `#3B82F6`),
accent `#2563EB` (hover `#1D4ED8`, selected fill at 15 % alpha), text
`#F8FAFC`/`#94A3B8`/`#64748B`, and status colours success `#10B981`, danger `#EF4444`,
warning `#F59E0B`, info `#38BDF8`. Small accent-coloured text uses `#60A5FA` for contrast.
The light palette maps the same roles onto slate-light tones.

| Asset | Location | Notes |
|-------|----------|-------|
| Text font | `assets/fonts/Inter-*.ttf` | Inter 4.1, Regular / Medium / SemiBold (OFL). See `assets/fonts/README.md`. |
| Icons | `assets/fonts/Phosphor*.ttf`, [`src/ui/icons.rs`](src/ui/icons.rs) | Phosphor Icons (MIT). Add new icons as constants in `icons.rs`. |
| Logo | `assets/logo/app-icon.png` | Window/taskbar icon and in-app logo. Embedded at build time. |

### Logo specification

Replace `assets/logo/app-icon.png` and rebuild (`cargo build --release`):

- **1024 × 1024 px PNG**, RGBA with a transparent background, sRGB.
- Keep the artwork inside the central **~824 × 824 px** (≈10 % padding on each side).
  This matches macOS/Windows icon grids and keeps it from looking cramped in the taskbar.
- It must still read clearly at **16, 24 and 32 px**. Prefer a bold, simple mark over
  fine detail or text.
- The header shows it at 28 pt (112 px on 4× displays) on both light and dark
  backgrounds, so avoid artwork that disappears on either one.
- Please also send the **SVG source** if you have one, so platform icon files (`.ico`,
  `.icns`) can be generated from it later.

## Not done yet / ideas

- **Formats that can't be decoded:** HEIC/HEIF, AVIF, RAW (CR2/NEF/ARW/DNG…) and SVG
  are found by the content and name methods. They get no thumbnails and are not
  included in *Similar-looking images*.
- **Videos:** these are matched by content, name or size only. There are no video
  thumbnails, and re-encoded videos are not detected as similar (that would need
  ffmpeg frame sampling).
- **Settings are not saved** between runs. The eframe `persistence` feature and serde
  would add this.
- **No hash cache:** every scan re-reads the files. A cache keyed by path, size and
  modified time would make repeat scans almost instant.
- **No undo** inside the app. Use *Move to Trash* or *Move to folder* if you may want
  to undo later.

## License

MIT, see [LICENSE](LICENSE). The bundled fonts keep their own licenses (see `assets/fonts/`).
