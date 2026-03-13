# xbrz-cli

A command-line tool for upscaling pixel art using the [xBRZ algorithm](https://en.wikipedia.org/wiki/Pixel-art_scaling_algorithms#xBR_family). Powered by the [xbrz-rs](https://crates.io/crates/xbrz-rs) crate.

## Installation

```sh
cargo install --git https://github.com/boekabart/xbrz-cli
```

Or from a local clone:

```sh
cargo build --release
```

The binary will be at `target/release/xbrz-cli`.

### Large LUT feature

For slightly faster scaling at the cost of a larger binary, enable the `large_lut` feature:

```sh
cargo build --release --features large_lut
```

## Usage

```
xbrz-cli <INPUT> <OUTPUT> [-f FACTOR] [-F FORMAT] [-v]
```

### Arguments

| Argument | Description |
|---|---|
| `INPUT` | Path to the input image |
| `OUTPUT` | Path to the output image |
| `-f`, `--factor` | Scale factor, 2-6 (default: 2) |
| `-F`, `--format` | Output format override (png, jpeg, bmp, webp, tiff, tga, gif, qoi) |
| `-v`, `--verbose` | Print dimensions and timing info to stderr |

### Examples

Scale a sprite 4x:

```sh
xbrz-cli sprite.png sprite_4x.png -f 4
```

Convert format while scaling:

```sh
xbrz-cli input.bmp output.webp -f 3
```

Override output format explicitly:

```sh
xbrz-cli input.png output.dat -F png -f 2
```

Verbose output:

```sh
xbrz-cli tile.png tile_big.png -f 6 -v
# Input:  16x16
# Output: 96x96 (factor 6)
# Done in 0.01s
```

## Supported Formats

PNG, JPEG, BMP, WebP, TIFF, TGA, GIF, QOI

> **Note:** JPEG does not support alpha transparency. When outputting to JPEG, any alpha channel will be lost.

## Architecture

Single-file binary crate (`src/main.rs`). No library component.

**Flow:** CLI args (clap) → image decode (image crate) → xBRZ upscale (xbrz-rs) → image encode & save.

**Key dependency:** `xbrz-rs` provides `scale_rgba(source, width, height, factor) -> Vec<u8>` for factors 1-6.

## Development

```sh
cargo clippy -- -D warnings        # Lint
cargo fmt --check                  # Format check
cargo fmt                          # Auto-format
```

## Acknowledgments

The actual xBRZ scaling is performed by [xbrz-rs](https://crates.io/crates/xbrz-rs), a Rust implementation of Zenju's [xBRZ algorithm](https://sourceforge.net/projects/xbrz/). All credit for the upscaling magic goes to them — this CLI is just a thin wrapper to make it easy to use from the command line.

## License

GPL-3.0-only — see [LICENSE](LICENSE).
