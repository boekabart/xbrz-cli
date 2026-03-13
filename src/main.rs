use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::Parser;
use image::ImageFormat;
use zune_core::bit_depth::BitDepth;
use zune_core::colorspace::ColorSpace;
use zune_core::options::EncoderOptions;
use zune_jpegxl::JxlSimpleEncoder;

#[derive(Parser)]
#[command(
    name = "xbrz-cli",
    version,
    about = "Pixel-art upscaling using the xBRZ algorithm"
)]
struct Args {
    /// Input image path
    input: PathBuf,

    /// Output image path [default: <input>_hq<N>.<ext>]
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Scale factor (2-6)
    #[arg(short = 'f', long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(2..=6))]
    factor: u8,

    /// Output format (png, jpeg, bmp, webp, tiff, tga, gif, qoi, jxl). Inferred from extension if omitted.
    #[arg(short = 'F', long = "format")]
    output_format: Option<String>,

    /// Print dimensions and timing info to stderr
    #[arg(short, long)]
    verbose: bool,
}

#[derive(PartialEq)]
enum OutputFormat {
    Image(ImageFormat),
    Jxl,
}

fn parse_format(name: &str) -> Option<OutputFormat> {
    match name.to_ascii_lowercase().as_str() {
        "png" => Some(OutputFormat::Image(ImageFormat::Png)),
        "jpg" | "jpeg" => Some(OutputFormat::Image(ImageFormat::Jpeg)),
        "bmp" => Some(OutputFormat::Image(ImageFormat::Bmp)),
        "webp" => Some(OutputFormat::Image(ImageFormat::WebP)),
        "tiff" | "tif" => Some(OutputFormat::Image(ImageFormat::Tiff)),
        "tga" => Some(OutputFormat::Image(ImageFormat::Tga)),
        "gif" => Some(OutputFormat::Image(ImageFormat::Gif)),
        "qoi" => Some(OutputFormat::Image(ImageFormat::Qoi)),
        "jxl" => Some(OutputFormat::Jxl),
        _ => None,
    }
}

fn default_output_path(input: &Path, factor: u8, format_override: &Option<String>) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = format_override
        .as_deref()
        .unwrap_or_else(|| input.extension().and_then(|e| e.to_str()).unwrap_or("png"));
    let name = format!("{stem}_hq{factor}.{ext}");
    input.with_file_name(name)
}

fn determine_format(output: &Path, format_override: &Option<String>) -> OutputFormat {
    if let Some(ref fmt) = format_override {
        return parse_format(fmt).unwrap_or_else(|| {
            eprintln!("Error: unknown output format '{fmt}'");
            std::process::exit(1);
        });
    }

    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_else(|| {
            eprintln!(
                "Error: cannot infer output format — no file extension. Use --format to specify."
            );
            std::process::exit(1);
        });

    parse_format(ext).unwrap_or_else(|| {
        eprintln!("Error: unknown output extension '.{ext}'. Use --format to specify.");
        std::process::exit(1);
    })
}

fn save_jxl(path: &Path, rgba: &[u8], width: usize, height: usize) -> Result<(), String> {
    let options = EncoderOptions::new(width, height, ColorSpace::RGBA, BitDepth::Eight);
    let encoder = JxlSimpleEncoder::new(rgba, options);
    let mut out = Vec::new();
    encoder.encode(&mut out).map_err(|e| format!("{e:?}"))?;
    fs::write(path, &out).map_err(|e| e.to_string())
}

fn main() {
    let args = Args::parse();
    let output = args
        .output
        .unwrap_or_else(|| default_output_path(&args.input, args.factor, &args.output_format));
    let format = determine_format(&output, &args.output_format);

    let start = Instant::now();

    let img = image::open(&args.input).unwrap_or_else(|e| {
        eprintln!("Error: failed to open '{}': {e}", args.input.display());
        std::process::exit(1);
    });

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    if args.verbose {
        eprintln!("Input:  {w}x{h}");
    }

    let factor = args.factor as usize;
    let scaled = xbrz::scale_rgba(&rgba, w as usize, h as usize, factor);

    let out_w = w as usize * factor;
    let out_h = h as usize * factor;

    if args.verbose {
        eprintln!("Output: {out_w}x{out_h} (factor {factor})");
    }

    match format {
        OutputFormat::Jxl => {
            save_jxl(&output, &scaled, out_w, out_h).unwrap_or_else(|e| {
                eprintln!("Error: failed to write '{}': {e}", output.display());
                std::process::exit(1);
            });
        }
        OutputFormat::Image(fmt) => {
            let is_jpeg = fmt == ImageFormat::Jpeg;

            if is_jpeg && args.verbose {
                eprintln!(
                    "Warning: JPEG does not support alpha channel — transparency will be lost"
                );
            }

            // JPEG doesn't support RGBA, so strip the alpha channel
            let (buf, color_type) = if is_jpeg {
                let rgb: Vec<u8> = scaled
                    .chunks_exact(4)
                    .flat_map(|px| &px[..3])
                    .copied()
                    .collect();
                (rgb, image::ColorType::Rgb8)
            } else {
                (scaled, image::ColorType::Rgba8)
            };

            image::save_buffer_with_format(
                &output,
                &buf,
                out_w as u32,
                out_h as u32,
                color_type,
                fmt,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: failed to write '{}': {e}", output.display());
                std::process::exit(1);
            });
        }
    }

    if args.verbose {
        let elapsed = start.elapsed();
        eprintln!("Done in {:.2}s", elapsed.as_secs_f64());
    }
}
