//! UTS 内置模块处理
//!
//! 处理 scan.uts.builtin_modules 列表，复制对应 AAR 文件并收集线上依赖

use std::collections::BTreeSet;
use std::path::Path;

/// 处理内置 UTS 模块：扫描 builtin_modules，复制 AAR，收集线上依赖
pub fn process_builtin_uts_modules(
    builtin_modules: &[crate::commands::resource::UtsBuiltinModule],
    sdk_libs: &Path,
    libs_dst: &Path,
    extra_deps: &mut BTreeSet<String>,
    window: &dyn crate::utils::process::BuildEventSink,
) -> Result<(), String> {
    // 复制 UTS 插件运行时基础 AAR
    crate::commands::android::artifacts::copy_optional_aar(
        sdk_libs,
        libs_dst,
        "utsplugin-release.aar",
        window,
    )?;

    for module in builtin_modules {
        crate::commands::android::artifacts::copy_optional_aar(
            sdk_libs,
            libs_dst,
            &module.local_aar,
            window,
        )?;
        for dep in &module.online_deps {
            extra_deps.insert(dep.clone());
        }
    }

    Ok(())
}
