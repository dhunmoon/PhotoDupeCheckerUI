# Bundled fonts

| File | Source | License |
|------|--------|---------|
| `Inter-Regular.ttf`, `Inter-Medium.ttf`, `Inter-SemiBold.ttf` | [Inter 4.1](https://github.com/rsms/inter) (`extras/ttf`) | SIL OFL 1.1, see `Inter-LICENSE.txt` |
| `Phosphor.ttf`, `Phosphor-Fill.ttf` | [Phosphor Icons](https://phosphoricons.com) via `egui-phosphor` 0.14 | MIT, see `Phosphor-LICENSE.txt` |

**The Inter files are modified:** their Private Use Area glyphs (U+E000–U+F8FF) were
removed, because Phosphor puts its icons in the same range. If Inter kept them, Inter
would be used first and icons would show up as random letters. To recreate the files:

```sh
pyftsubset Inter-Regular.ttf --unicodes="U+0000-DFFF,U+F900-10FFFF" \
  --layout-features='*' --name-IDs='*' --name-languages='*' --glyph-names \
  --notdef-outline --output-file=Inter-Regular.ttf
```
