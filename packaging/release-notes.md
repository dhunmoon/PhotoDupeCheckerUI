## Downloads

| System | File |
|---|---|
| **Windows 10/11** (64-bit) | `…-windows-x86_64-setup.exe` (installer) or `…-windows-x86_64.zip` (portable) |
| **macOS, Apple Silicon** (M1/M2/M3/M4) | `…-macos-apple-silicon.dmg` |
| **macOS, Intel** | `…-macos-intel.dmg` |
| **Ubuntu / Debian / Mint / Pop!_OS** | `…_amd64.deb` |
| **Fedora / RHEL / Rocky / openSUSE** | `…x86_64.rpm` |
| **Any Linux** (Arch, etc.) | `…-linux-x86_64.AppImage` or `…-linux-x86_64.tar.gz` |

Not sure which Mac you have? Apple menu → *About This Mac* → "Chip: Apple …" means Apple Silicon, "Processor: Intel …" means Intel.

### First launch (the app is not code-signed yet)

- **Windows:** if SmartScreen says *"Windows protected your PC"*, click **More info → Run anyway**.
- **macOS:** open the `.dmg`, drag **Dupe Finder** to **Applications**, then open it. If macOS says it *"cannot be opened"* or *"could not verify"*, go to **System Settings → Privacy & Security** and click **Open Anyway** (or run `xattr -dr com.apple.quarantine "/Applications/Dupe Finder.app"` in Terminal).
- **Linux AppImage:** `chmod +x dupe-finder-*.AppImage` then run it.

Verify a download with `SHA256SUMS.txt` (e.g. `sha256sum -c SHA256SUMS.txt --ignore-missing`).
