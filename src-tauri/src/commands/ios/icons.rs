//! iOS AppIcon.appiconset 生成。

use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
struct IosAppIconSlot {
    manifest_key: &'static str,
    filename: &'static str,
    idiom: &'static str,
    size: &'static str,
    scale: &'static str,
    pixels: u32,
}

const IOS_APP_ICON_SLOTS: &[IosAppIconSlot] = &[
    IosAppIconSlot {
        manifest_key: "iphone.notification@2x",
        filename: "Icon-iphone-20@2x.png",
        idiom: "iphone",
        size: "20x20",
        scale: "2x",
        pixels: 40,
    },
    IosAppIconSlot {
        manifest_key: "iphone.notification@3x",
        filename: "Icon-iphone-20@3x.png",
        idiom: "iphone",
        size: "20x20",
        scale: "3x",
        pixels: 60,
    },
    IosAppIconSlot {
        manifest_key: "iphone.settings@2x",
        filename: "Icon-iphone-29@2x.png",
        idiom: "iphone",
        size: "29x29",
        scale: "2x",
        pixels: 58,
    },
    IosAppIconSlot {
        manifest_key: "iphone.settings@3x",
        filename: "Icon-iphone-29@3x.png",
        idiom: "iphone",
        size: "29x29",
        scale: "3x",
        pixels: 87,
    },
    IosAppIconSlot {
        manifest_key: "iphone.spotlight@2x",
        filename: "Icon-iphone-40@2x.png",
        idiom: "iphone",
        size: "40x40",
        scale: "2x",
        pixels: 80,
    },
    IosAppIconSlot {
        manifest_key: "iphone.spotlight@3x",
        filename: "Icon-iphone-40@3x.png",
        idiom: "iphone",
        size: "40x40",
        scale: "3x",
        pixels: 120,
    },
    IosAppIconSlot {
        manifest_key: "iphone.app@2x",
        filename: "Icon-iphone-60@2x.png",
        idiom: "iphone",
        size: "60x60",
        scale: "2x",
        pixels: 120,
    },
    IosAppIconSlot {
        manifest_key: "iphone.app@3x",
        filename: "Icon-iphone-60@3x.png",
        idiom: "iphone",
        size: "60x60",
        scale: "3x",
        pixels: 180,
    },
    IosAppIconSlot {
        manifest_key: "ipad.notification",
        filename: "Icon-ipad-20.png",
        idiom: "ipad",
        size: "20x20",
        scale: "1x",
        pixels: 20,
    },
    IosAppIconSlot {
        manifest_key: "ipad.notification@2x",
        filename: "Icon-ipad-20@2x.png",
        idiom: "ipad",
        size: "20x20",
        scale: "2x",
        pixels: 40,
    },
    IosAppIconSlot {
        manifest_key: "ipad.settings",
        filename: "Icon-ipad-29.png",
        idiom: "ipad",
        size: "29x29",
        scale: "1x",
        pixels: 29,
    },
    IosAppIconSlot {
        manifest_key: "ipad.settings@2x",
        filename: "Icon-ipad-29@2x.png",
        idiom: "ipad",
        size: "29x29",
        scale: "2x",
        pixels: 58,
    },
    IosAppIconSlot {
        manifest_key: "ipad.spotlight",
        filename: "Icon-ipad-40.png",
        idiom: "ipad",
        size: "40x40",
        scale: "1x",
        pixels: 40,
    },
    IosAppIconSlot {
        manifest_key: "ipad.spotlight@2x",
        filename: "Icon-ipad-40@2x.png",
        idiom: "ipad",
        size: "40x40",
        scale: "2x",
        pixels: 80,
    },
    IosAppIconSlot {
        manifest_key: "ipad.app",
        filename: "Icon-ipad-76.png",
        idiom: "ipad",
        size: "76x76",
        scale: "1x",
        pixels: 76,
    },
    IosAppIconSlot {
        manifest_key: "ipad.app@2x",
        filename: "Icon-ipad-76@2x.png",
        idiom: "ipad",
        size: "76x76",
        scale: "2x",
        pixels: 152,
    },
    IosAppIconSlot {
        manifest_key: "ipad.proapp@2x",
        filename: "Icon-ipad-83.5@2x.png",
        idiom: "ipad",
        size: "83.5x83.5",
        scale: "2x",
        pixels: 167,
    },
    IosAppIconSlot {
        manifest_key: "appstore",
        filename: "Icon-1024.png",
        idiom: "ios-marketing",
        size: "1024x1024",
        scale: "1x",
        pixels: 1024,
    },
];

pub(super) fn generate_app_icons(
    project_root: &Path,
    config: &crate::commands::project::ProjectConfig,
    manifest_info: Option<&crate::commands::resource::UniappManifestInfo>,
) -> Result<(), String> {
    let manifest_icons = manifest_info.and_then(|info| info.ios_icons.as_ref());
    let fallback_source = manifest_icons
        .and_then(|icons| icons.ios.get("appstore"))
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .or_else(|| {
            let source = config.app.icon1024.trim();
            (!source.is_empty())
                .then(|| PathBuf::from(source))
                .filter(|path| path.exists())
        });
    if manifest_icons.is_none() && fallback_source.is_none() {
        return Ok(());
    }

    let appicon = find_dir_named(project_root, "AppIcon.appiconset")
        .unwrap_or_else(|| project_root.join("Assets.xcassets/AppIcon.appiconset"));
    crate::utils::fs::ensure_directory(&appicon).map_err(|e| e.to_string())?;
    let fallback_image = fallback_source
        .as_ref()
        .map(|source| image::open(source).map(|image| image.to_rgba8()))
        .transpose()
        .map_err(|e| format!("读取 iOS 图标源失败: {}", e))?;

    for slot in IOS_APP_ICON_SLOTS {
        if let Some(source) = manifest_icons
            .and_then(|icons| icons.ios.get(slot.manifest_key))
            .map(PathBuf::from)
            .filter(|source| source.exists())
        {
            std::fs::copy(&source, appicon.join(slot.filename))
                .map_err(|e| format!("复制 iOS 图标失败 {}: {}", source.display(), e))?;
            continue;
        }
        let Some(img) = fallback_image.as_ref() else {
            continue;
        };
        let resized =
            image::imageops::resize(img, slot.pixels, slot.pixels, image::imageops::Lanczos3);
        resized
            .save(appicon.join(slot.filename))
            .map_err(|e| format!("生成 iOS 图标失败: {}", e))?;
    }
    write_appicon_contents(&appicon)
}

fn write_appicon_contents(appicon: &Path) -> Result<(), String> {
    let images = IOS_APP_ICON_SLOTS
        .iter()
        .map(|slot| {
            serde_json::json!({
                "idiom": slot.idiom,
                "size": slot.size,
                "scale": slot.scale,
                "filename": slot.filename
            })
        })
        .collect::<Vec<_>>();
    let contents =
        serde_json::json!({ "images": images, "info": { "author": "unipack-tool", "version": 1 } });
    let json = serde_json::to_string_pretty(&contents).map_err(|e| e.to_string())?;
    std::fs::write(appicon.join("Contents.json"), json)
        .map_err(|e| format!("写入 AppIcon Contents.json 失败: {}", e))
}

fn find_dir_named(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                return Some(path);
            }
            if is_xcode_package_dir(&path) {
                continue;
            }
            if let Some(found) = find_dir_named(&path, name) {
                return Some(found);
            }
        }
    }
    None
}

fn is_xcode_package_dir(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("framework" | "xcframework" | "bundle" | "xcodeproj" | "xcworkspace")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appicon_contents_includes_marketing_icon() {
        let dir =
            std::env::temp_dir().join(format!("unipack-ios-appicon-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        write_appicon_contents(&dir).unwrap();
        let content = std::fs::read_to_string(dir.join("Contents.json")).unwrap();
        assert!(content.contains("ios-marketing"));
        assert!(content.contains("Icon-1024.png"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
