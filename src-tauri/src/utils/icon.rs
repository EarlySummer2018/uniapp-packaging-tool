#![allow(dead_code)]
use anyhow::Result;
use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use std::path::Path;

pub struct IconSize {
    pub width: u32,
    pub height: u32,
    pub suffix: &'static str,
}

pub const ANDROID_ICON_SIZES: &[IconSize] = &[
    IconSize {
        width: 48,
        height: 48,
        suffix: "-mipmap-mdpi",
    },
    IconSize {
        width: 72,
        height: 72,
        suffix: "-mipmap-hdpi",
    },
    IconSize {
        width: 96,
        height: 96,
        suffix: "-mipmap-xhdpi",
    },
    IconSize {
        width: 144,
        height: 144,
        suffix: "-mipmap-xxhdpi",
    },
    IconSize {
        width: 192,
        height: 192,
        suffix: "-mipmap-xxxhdpi",
    },
];

pub const IOS_ICON_SIZES: &[IconSize] = &[
    IconSize {
        width: 20,
        height: 20,
        suffix: "-29",
    },
    IconSize {
        width: 40,
        height: 40,
        suffix: "-29@2x",
    },
    IconSize {
        width: 60,
        height: 60,
        suffix: "-29@3x",
    },
    IconSize {
        width: 58,
        height: 58,
        suffix: "-40",
    },
    IconSize {
        width: 80,
        height: 80,
        suffix: "-40@2x",
    },
    IconSize {
        width: 120,
        height: 120,
        suffix: "-40@3x",
    },
    IconSize {
        width: 60,
        height: 60,
        suffix: "-60",
    },
    IconSize {
        width: 120,
        height: 120,
        suffix: "-60@2x",
    },
    IconSize {
        width: 180,
        height: 180,
        suffix: "-60@3x",
    },
];

pub fn generate_icons_from_source(
    source_path: &Path,
    output_dir: &Path,
    sizes: &[IconSize],
    prefix: &str,
) -> Result<Vec<IconOutput>> {
    let img = image::open(source_path)?;
    let rgba = img.to_rgba8();
    let mut outputs = Vec::new();

    for size in sizes {
        let resized =
            image::imageops::resize(&rgba, size.width, size.height, image::imageops::Lanczos3);

        let file_name = format!("{}{}.png", prefix, size.suffix);
        let output_path = output_dir.join(&file_name);

        let mut out_file = std::fs::File::create(&output_path)?;
        let encoder = PngEncoder::new(&mut out_file);
        encoder.write_image(
            resized.as_raw(),
            size.width,
            size.height,
            image::ExtendedColorType::Rgba8,
        )?;

        outputs.push(IconOutput {
            path: output_path,
            width: size.width,
            height: size.height,
            suffix: size.suffix.to_string(),
        });
    }

    Ok(outputs)
}

#[derive(Debug)]
pub struct IconOutput {
    pub path: std::path::PathBuf,
    pub width: u32,
    pub height: u32,
    pub suffix: String,
}

pub fn generate_android_icons(source_path: &Path, res_dir: &Path) -> Result<Vec<IconOutput>> {
    generate_icons_from_source(source_path, res_dir, ANDROID_ICON_SIZES, "ic_launcher")
}

pub fn generate_ios_icons(source_path: &Path, assets_dir: &Path) -> Result<Vec<IconOutput>> {
    generate_icons_from_source(source_path, assets_dir, IOS_ICON_SIZES, "AppIcon")
}

pub fn generate_adaptive_icons(
    source_path: &Path,
    foreground_path: &Path,
    res_dir: &Path,
    bg_color: [u8; 4],
) -> Result<Vec<IconOutput>> {
    let fg_img = image::open(foreground_path).unwrap_or_else(|_| image::open(source_path).unwrap());
    let mut results = Vec::new();

    for size in ANDROID_ICON_SIZES {
        let mut canvas =
            image::ImageBuffer::from_fn(size.width, size.height, |_, _| image::Rgba(bg_color));

        let scale = (size.width as f32 * 0.66) / fg_img.width() as f32;
        let new_w = (fg_img.width() as f32 * scale) as u32;
        let new_h = (fg_img.height() as f32 * scale) as u32;
        let resized =
            image::imageops::resize(&fg_img.to_rgba8(), new_w, new_h, image::imageops::Lanczos3);

        let offset_x = ((size.width - new_w) / 2) as i64;
        let offset_y = ((size.height - new_h) / 2) as i64;
        image::imageops::overlay(&mut canvas, &resized, offset_x, offset_y);

        let file_name = format!("ic_launcher_foreground{}.png", size.suffix);
        let output_path = res_dir.join(&file_name);

        let mut out_file = std::fs::File::create(&output_path)?;
        let encoder = PngEncoder::new(&mut out_file);
        encoder.write_image(
            canvas.as_raw(),
            size.width,
            size.height,
            image::ExtendedColorType::Rgba8,
        )?;

        results.push(IconOutput {
            path: output_path,
            width: size.width,
            height: size.height,
            suffix: size.suffix.to_string(),
        });
    }

    Ok(results)
}
