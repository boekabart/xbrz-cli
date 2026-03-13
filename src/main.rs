use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::Parser;
use image::ImageFormat;

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

    /// Output format (png, jpeg, bmp, webp, tiff, tga, gif, qoi). Inferred from extension if omitted.
    #[arg(short = 'F', long = "format")]
    output_format: Option<String>,

    /// Print dimensions and timing info to stderr
    #[arg(short, long)]
    verbose: bool,
}

fn parse_format(name: &str) -> Option<ImageFormat> {
    match name.to_ascii_lowercase().as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "bmp" => Some(ImageFormat::Bmp),
        "webp" => Some(ImageFormat::WebP),
        "tiff" | "tif" => Some(ImageFormat::Tiff),
        "tga" => Some(ImageFormat::Tga),
        "gif" => Some(ImageFormat::Gif),
        "qoi" => Some(ImageFormat::Qoi),
        _ => None,
    }
}

fn default_output_path(input: &Path, factor: u8) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = input.extension().and_then(|e| e.to_str()).unwrap_or("png");
    let name = format!("{stem}_hq{factor}.{ext}");
    input.with_file_name(name)
}

fn determine_format(output: &Path, format_override: &Option<String>) -> ImageFormat {
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

fn main() {
    let args = Args::parse();
    let output = args
        .output
        .unwrap_or_else(|| default_output_path(&args.input, args.factor));
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

    let is_jpeg = format == ImageFormat::Jpeg;

    if is_jpeg && args.verbose {
        eprintln!("Warning: JPEG does not support alpha channel — transparency will be lost");
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
        format,
    )
    .unwrap_or_else(|e| {
        eprintln!("Error: failed to write '{}': {e}", output.display());
        std::process::exit(1);
    });

    if args.verbose {
        let elapsed = start.elapsed();
        eprintln!("Done in {:.2}s", elapsed.as_secs_f64());
    }
}
