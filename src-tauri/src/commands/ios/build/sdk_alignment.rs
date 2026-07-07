use std::path::{Path, PathBuf};
use std::process::Command;

use super::sdk_support::IosSdkSupportLog;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LipoArchitectureSlice {
    name: String,
    offset: usize,
}

pub(super) fn ios_static_library_needs_alignment_repair(path: &Path) -> Result<bool, String> {
    let slices = lipo_architecture_slices(path)?;
    let work_dir = std::env::temp_dir().join(format!(
        "unipack-ios-align-check-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&work_dir).map_err(|e| {
        format!(
            "创建 iOS 静态库检测临时目录失败 {}: {}",
            work_dir.display(),
            e
        )
    })?;
    let result = (|| {
        if slices.is_empty() {
            return archive_has_unaligned_macho_members(path);
        }
        for slice in slices {
            let thin = work_dir.join(format!("{}.a", slice.name));
            run_command(
                Command::new("xcrun")
                    .arg("lipo")
                    .arg(path)
                    .arg("-thin")
                    .arg(&slice.name)
                    .arg("-output")
                    .arg(&thin),
                &format!("提取 iOS 静态库架构 {} 失败", slice.name),
            )?;
            if archive_has_unaligned_macho_members_with_base_offset(&thin, slice.offset)? {
                return Ok(true);
            }
        }
        Ok(false)
    })();
    let _ = std::fs::remove_dir_all(&work_dir);
    result
}

pub(super) fn repair_ios_static_library_alignment(
    path: &Path,
    logs: &mut Vec<IosSdkSupportLog>,
) -> Result<(), String> {
    let library_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_string();
    logs.push(IosSdkSupportLog::info(format!(
        "开始修复 iOS 静态库对齐: {}",
        library_name
    )));
    make_file_writable(path)?;

    let backup = path.with_extension("a.bak");
    if backup.exists() {
        logs.push(IosSdkSupportLog::info(format!(
            "{} 备份已存在，跳过备份",
            library_name
        )));
    } else {
        std::fs::copy(path, &backup).map_err(|e| {
            format!(
                "备份 iOS 静态库失败 {} -> {}: {}",
                path.display(),
                backup.display(),
                e
            )
        })?;
        logs.push(IosSdkSupportLog::info(format!(
            "已备份 workspace 副本: {}",
            backup.display()
        )));
    }

    let archs = lipo_architectures(path)?;
    if archs.is_empty() {
        return Err(format!(
            "无法识别 iOS 静态库架构，跳过修复: {}",
            path.display()
        ));
    }
    logs.push(IosSdkSupportLog::info(format!(
        "{} 包含架构: {}",
        library_name,
        archs.join("、")
    )));

    let work_dir = std::env::temp_dir().join(format!(
        "unipack-ios-align-fix-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&work_dir).map_err(|e| {
        format!(
            "创建 iOS 静态库修复临时目录失败 {}: {}",
            work_dir.display(),
            e
        )
    })?;
    let result = (|| {
        let mut fixed_libraries = Vec::new();
        for arch in &archs {
            logs.push(IosSdkSupportLog::info(format!(
                "{} 处理架构: {}",
                library_name, arch
            )));
            let arch_dir = work_dir.join(arch);
            std::fs::create_dir_all(&arch_dir).map_err(|e| {
                format!(
                    "创建 iOS 静态库架构工作目录失败 {}: {}",
                    arch_dir.display(),
                    e
                )
            })?;
            let thin = arch_dir.join("thin.a");
            run_command(
                Command::new("xcrun")
                    .arg("lipo")
                    .arg(path)
                    .arg("-thin")
                    .arg(arch)
                    .arg("-output")
                    .arg(&thin),
                &format!("提取 iOS 静态库架构 {} 失败", arch),
            )?;
            run_command(
                Command::new("xcrun")
                    .arg("ar")
                    .arg("x")
                    .arg("thin.a")
                    .current_dir(&arch_dir),
                &format!("解包 iOS 静态库架构 {} 失败", arch),
            )?;
            let _ = std::fs::remove_file(&thin);
            let _ = std::fs::remove_file(arch_dir.join("__.SYMDEF"));
            let _ = std::fs::remove_file(arch_dir.join("__.SYMDEF SORTED"));

            let object_files = sorted_object_files(&arch_dir)?;
            if object_files.is_empty() {
                return Err(format!(
                    "iOS 静态库架构 {} 未解出 .o 文件: {}",
                    arch,
                    path.display()
                ));
            }
            let fixed = arch_dir.join("fixed.a");
            let libtool_result = run_libtool_static(&fixed, &object_files);
            if libtool_result.is_err() {
                run_ar_static(&fixed, &object_files)?;
            }
            fixed_libraries.push(fixed);
        }

        run_lipo_create(&fixed_libraries, path)?;
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&work_dir);
    result?;

    if ios_static_library_needs_alignment_repair(path)? {
        return Err(format!(
            "iOS 静态库修复后仍存在 not 8-byte aligned 风险: {}",
            path.display()
        ));
    }

    logs.push(IosSdkSupportLog::success(format!(
        "{} 修复完成",
        library_name
    )));
    Ok(())
}

fn make_file_writable(path: &Path) -> Result<(), String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("读取文件权限失败 {}: {}", path.display(), e))?;
    let mut permissions = metadata.permissions();
    if permissions.readonly() {
        permissions.set_readonly(false);
        std::fs::set_permissions(path, permissions)
            .map_err(|e| format!("设置文件可写失败 {}: {}", path.display(), e))?;
    }
    Ok(())
}

fn lipo_architectures(path: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("xcrun")
        .arg("lipo")
        .arg("-info")
        .arg(path)
        .output()
        .map_err(|e| format!("执行 xcrun lipo -info 失败 {}: {}", path.display(), e))?;
    if !output.status.success() {
        return Err(format!(
            "读取 iOS 静态库架构失败 {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(parse_lipo_architectures(&text))
}

fn parse_lipo_architectures(text: &str) -> Vec<String> {
    let line = text.trim();
    let archs = line
        .split(" are: ")
        .nth(1)
        .or_else(|| line.split(" is architecture: ").nth(1))
        .unwrap_or_default();
    archs
        .split_whitespace()
        .map(str::trim)
        .filter(|arch| !arch.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn lipo_architecture_slices(path: &Path) -> Result<Vec<LipoArchitectureSlice>, String> {
    let output = Command::new("xcrun")
        .arg("lipo")
        .arg("-detailed_info")
        .arg(path)
        .output()
        .map_err(|e| {
            format!(
                "执行 xcrun lipo -detailed_info 失败 {}: {}",
                path.display(),
                e
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "读取 iOS 静态库架构详情失败 {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut slices = parse_lipo_architecture_slices(&text);
    if slices.is_empty() {
        slices = parse_lipo_architectures(&text)
            .into_iter()
            .map(|name| LipoArchitectureSlice { name, offset: 0 })
            .collect();
    }
    Ok(slices)
}

fn parse_lipo_architecture_slices(text: &str) -> Vec<LipoArchitectureSlice> {
    let mut slices = Vec::new();
    let mut current_arch: Option<String> = None;
    for line in text.lines().map(str::trim) {
        if let Some(arch) = line.strip_prefix("architecture ") {
            current_arch = Some(arch.trim().to_string());
            continue;
        }
        let Some(offset_text) = line.strip_prefix("offset ") else {
            continue;
        };
        let Some(name) = current_arch.take() else {
            continue;
        };
        if let Ok(offset) = offset_text.trim().parse::<usize>() {
            slices.push(LipoArchitectureSlice { name, offset });
        }
    }
    slices
}

fn archive_has_unaligned_macho_members(path: &Path) -> Result<bool, String> {
    archive_has_unaligned_macho_members_with_base_offset(path, 0)
}

fn archive_has_unaligned_macho_members_with_base_offset(
    path: &Path,
    base_offset: usize,
) -> Result<bool, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("读取 iOS 静态库失败 {}: {}", path.display(), e))?;
    if !bytes.starts_with(b"!<arch>\n") {
        return Ok(false);
    }

    let mut offset = 8usize;
    while offset + 60 <= bytes.len() {
        let header = &bytes[offset..offset + 60];
        if &header[58..60] != b"`\n" {
            return Err(format!("iOS 静态库 ar header 异常: {}", path.display()));
        }
        let size_text = std::str::from_utf8(&header[48..58])
            .map_err(|e| format!("解析 iOS 静态库 member size 失败 {}: {}", path.display(), e))?
            .trim();
        let size = size_text
            .parse::<usize>()
            .map_err(|e| format!("解析 iOS 静态库 member size 失败 {}: {}", path.display(), e))?;

        let data_start = offset + 60;
        if data_start + size > bytes.len() {
            return Err(format!("iOS 静态库 member 越界: {}", path.display()));
        }

        let name = std::str::from_utf8(&header[..16])
            .unwrap_or_default()
            .trim();
        let name_len = name
            .strip_prefix("#1/")
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or_default();
        let object_start = data_start + name_len;
        if object_start + 4 <= data_start + size
            && is_64_bit_macho_magic(&bytes[object_start..object_start + 4])
            && (base_offset + object_start) % 8 != 0
        {
            return Ok(true);
        }

        offset = data_start + size;
        if offset % 2 == 1 {
            offset += 1;
        }
    }
    Ok(false)
}

fn is_64_bit_macho_magic(bytes: &[u8]) -> bool {
    matches!(bytes, [0xfe, 0xed, 0xfa, 0xcf] | [0xcf, 0xfa, 0xed, 0xfe])
}

fn sorted_object_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = std::fs::read_dir(dir)
        .map_err(|e| format!("读取 iOS 静态库工作目录失败 {}: {}", dir.display(), e))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("o"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn run_libtool_static(output: &Path, objects: &[PathBuf]) -> Result<(), String> {
    let mut command = Command::new("xcrun");
    command.arg("libtool").arg("-static").arg("-o").arg(output);
    for object in objects {
        command.arg(object);
    }
    run_command(&mut command, "使用 libtool 重建 iOS 静态库失败")
}

fn run_ar_static(output: &Path, objects: &[PathBuf]) -> Result<(), String> {
    let mut command = Command::new("xcrun");
    command.arg("ar").arg("rcs").arg(output);
    for object in objects {
        command.arg(object);
    }
    run_command(&mut command, "使用 ar 重建 iOS 静态库失败")
}

fn run_lipo_create(inputs: &[PathBuf], output: &Path) -> Result<(), String> {
    let mut command = Command::new("xcrun");
    command.arg("lipo").arg("-create");
    for input in inputs {
        command.arg(input);
    }
    command.arg("-output").arg(output);
    run_command(&mut command, "合并 iOS 静态库架构失败")
}

fn run_command(command: &mut Command, context: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|e| format!("{}: {}", context, e))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("{}: {}", context, stderr.trim()))
}

#[cfg(test)]
mod tests {
    use super::{
        archive_has_unaligned_macho_members, archive_has_unaligned_macho_members_with_base_offset,
        parse_lipo_architecture_slices, parse_lipo_architectures, LipoArchitectureSlice,
    };

    #[test]
    fn parses_lipo_architectures_for_fat_and_thin_libraries() {
        assert_eq!(
            parse_lipo_architectures(
                "Architectures in the fat file: lib.a are: armv7 x86_64 arm64"
            ),
            vec!["armv7", "x86_64", "arm64"]
        );
        assert_eq!(
            parse_lipo_architectures("Non-fat file: lib.a is architecture: arm64"),
            vec!["arm64"]
        );
    }

    #[test]
    fn parses_lipo_detailed_info_offsets() {
        let slices = parse_lipo_architecture_slices(
            r#"Fat header in: lib.a
fat_magic 0xcafebabe
nfat_arch 2
architecture x86_64
    cputype CPU_TYPE_X86_64
    offset 787976
    size 293328
    align 2^3 (8)
architecture arm64
    cputype CPU_TYPE_ARM64
    offset 1081304
    size 320032
    align 2^3 (8)
"#,
        );

        assert_eq!(
            slices,
            vec![
                LipoArchitectureSlice {
                    name: "x86_64".to_string(),
                    offset: 787976
                },
                LipoArchitectureSlice {
                    name: "arm64".to_string(),
                    offset: 1081304
                }
            ]
        );
    }

    #[test]
    fn detects_unaligned_macho_member_in_archive() {
        let root =
            std::env::temp_dir().join(format!("unipack-ios-unaligned-ar-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let archive = root.join("libBroken.a");
        let mut bytes = b"!<arch>\n".to_vec();
        bytes.extend_from_slice(ar_header("#1/3", 7).as_bytes());
        bytes.extend_from_slice(b"foo");
        bytes.extend_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
        if bytes.len() % 2 == 1 {
            bytes.push(b'\n');
        }
        std::fs::write(&archive, bytes).unwrap();

        assert!(archive_has_unaligned_macho_members(&archive).unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn detects_fat_slice_base_offset_alignment_for_64_bit_members() {
        let root = std::env::temp_dir().join(format!(
            "unipack-ios-fat-offset-ar-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let archive = root.join("libFatOffset.a");
        let mut bytes = b"!<arch>\n".to_vec();
        bytes.extend_from_slice(ar_header("#1/20", 24).as_bytes());
        bytes.extend_from_slice(b"AlignedObjectFile.o ");
        bytes.extend_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
        std::fs::write(&archive, bytes).unwrap();

        assert!(!archive_has_unaligned_macho_members_with_base_offset(&archive, 0).unwrap());
        assert!(archive_has_unaligned_macho_members_with_base_offset(&archive, 4).unwrap());
        let _ = std::fs::remove_dir_all(root);
    }

    fn ar_header(name: &str, size: usize) -> String {
        format!(
            "{:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
            name, 0, 0, 0, 0o100644, size
        )
    }
}
