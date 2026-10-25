## Downloads

Pick the file for your system. Every file is named `DupeFinder-<version>-<system>-<cpu>`.

### Windows 10 / 11

| Your PC | Installer | Portable (no install) |
|---|---|---|
| Most PCs (64-bit Intel/AMD) | `…-windows-x64-setup.exe` | `…-windows-x64-portable.zip` |
| ARM PCs (Snapdragon, Surface Pro X …) | `…-windows-arm64-setup.exe` | `…-windows-arm64-portable.zip` |
| Old 32-bit Windows | `…-windows-x86-setup.exe` | `…-windows-x86-portable.zip` |

Not sure? Use **x64**. To check: *Settings → System → About → System type*.

### macOS 11 or newer

| Your Mac | File |
|---|---|
| Apple Silicon (M1, M2, M3, M4 …) | `…-macos-apple-silicon.dmg` |
| Intel | `…-macos-intel.dmg` |

Not sure? Apple menu → *About This Mac*: "Chip: Apple …" means Apple Silicon, "Processor: Intel …" means Intel.

### Linux (x64 and arm64)

| Distribution | File | Install |
|---|---|---|
| Ubuntu, Debian, Mint, Pop!_OS, elementary | `…-linux-<cpu>.deb` | `sudo apt install ./DupeFinder-*.deb` |
| Fedora, RHEL, Rocky, Alma, openSUSE | `…-linux-<cpu>.rpm` | `sudo dnf install ./DupeFinder-*.rpm` (openSUSE: `sudo zypper install`) |
| Arch, Manjaro, EndeavourOS | `…-linux-<cpu>.pkg.tar.zst` | `sudo pacman -U DupeFinder-*.pkg.tar.zst` |
| Any other distro | `…-linux-<cpu>.AppImage` | `chmod +x DupeFinder-*.AppImage`, then run it |
| Manual install | `…-linux-<cpu>.tar.gz` | extract and run `dupe_finder` |

`<cpu>` is **x64** for most PCs and **arm64** for ARM machines (Raspberry Pi 4/5 with a 64-bit OS, Ampere, Snapdragon laptops). Check with `uname -m`: `x86_64` → x64, `aarch64` → arm64. Requires glibc 2.28+ (any distro from 2018 onward).

### First launch (the app is not code-signed yet)

- **Windows:** if SmartScreen says *"Windows protected your PC"*, click **More info → Run anyway**.
- **macOS:** open the `.dmg`, drag **Dupe Finder** to **Applications**, then open it. If macOS says it *"cannot be opened"* or *"could not verify"*, go to **System Settings → Privacy & Security** and click **Open Anyway** (or run `xattr -dr com.apple.quarantine "/Applications/Dupe Finder.app"` in Terminal).

Verify a download with `SHA256SUMS.txt` (e.g. `sha256sum -c SHA256SUMS.txt --ignore-missing`).
