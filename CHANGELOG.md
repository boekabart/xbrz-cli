# Changelog

## [0.2.0] - 2026-03-13

### Changed
- Output is now specified via `-o`/`--output` instead of a positional argument.
- When omitted, the output path defaults to `<input>_hq<N>.<ext>` (e.g. `sprite.png` → `sprite_hq2.png`).

### Added
- `cargo install xbrz-cli` support via crates.io metadata.
- CI publishes to crates.io on tagged releases.
- AI assistant instruction files (CLAUDE.md, .cursorrules, .windsurfrules, copilot-instructions.md).

## [0.1.0] - 2026-03-13

Initial release.

- xBRZ pixel-art upscaling (factors 2–6) via the xbrz-rs crate.
- Supports PNG, JPEG, BMP, WebP, TIFF, TGA, GIF, QOI.
- Automatic format inference from file extension, with `--format` override.
- JPEG alpha channel stripping.
- Verbose mode (`-v`) with input/output dimensions and timing.
- Optional `large_lut` feature for faster scaling.
