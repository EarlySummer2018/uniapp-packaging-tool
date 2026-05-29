# UniApp Pack Tool 修复计划

## Summary

- 结论：当前项目只是可编译原型，并未按 `uniapp-pack-tool-plan.md` 完整开发。`npm run build` 与 `cargo check` 通过，但打包主流程没有闭环。
- 主要偏差：前后端项目配置割裂；Tauri 插件/权限未注册完整；构建中心传文件名而非真实资源路径；Android 构建硬编码 SDK/AppKey/图标/输出目录且缺 UTS；模板缺 `gradlew`；iOS/Harmony 只是泛化外部项目构建。
- 交付物：新增 `/Applications/project/tauri/uniapp-pack-tool-fix-plan.md`，写入本修复计划，再按以下阶段修复。

## Public Interfaces / Types

- 以 Rust 后端为配置唯一来源，保存到 `~/.unipack/projects/{id}/config.json`，字段对齐原方案：`app`、`android`、`ios`、`harmony`、`outputDir`。
- 新增/调整命令：`import_uniapp_resource(project_id, resource_path)`、`save_project_config`、`build_android_apk`、`build_ios_ipa`、`build_harmony_hap`、签名密码保存/删除/检测命令。
- 新增结构：`ResourceScanResult`、`UtsPluginScanResult`、`BuildArtifact`、`BuildLogEvent { build_id, platform, level, message, progress }`。
- 迁移当前 Pinia/plugin-store 和 `~/.config/unipack-tool/projects/*.json` 旧数据；Pinia 只做 UI 缓存和当前选择状态。

## Implementation Changes

- 注册 Tauri `store`、`dialog`、`fs`、`shell` 等所需插件与 capabilities；前端文件选择改用官方 dialog API，替换当前 `invoke('open')`。
- 项目配置页补齐 SDK 路径、DCloud AppKey、应用名、版本号、1024 图标、输出目录、Android/iOS/Harmony 签名与导出配置。
- 资源导入支持真实 `__UNI__*` 文件夹和 zip，解析 `manifest.json`，校验 SDK 与 HBuilderX 版本，扫描 `uni_modules`，展示 UTS 基础运行时、内置模块、自定义插件摘要。
- Android：修复 `bundled/android-template`，加入 Gradle wrapper，渲染 `.tmpl` 为真实文件；使用配置的 SDK/AppKey/icon/outputDir/versionCode/signing secrets；复制方案要求的 6 个 AAR、`assets/data`、UniApp 资源；更新 `dcloud_control.xml`；生成图标；注入 UTS runtime/内置模块/自定义模块；执行 `./gradlew assembleRelease --stacktrace`，复制 APK 并记录历史。
- iOS：新增完整 `build_ios_ipa`，复制 `{ios.sdkPath}/HBuilder-Hello`，修改 `Info.plist`/`project.pbxproj`，导入资源和图标，安装描述文件，生成 `ExportOptions.plist`，执行 archive/exportArchive，处理 UTS frameworks，复制 IPA；非 macOS 禁用。
- Harmony：替换当前泛化 `hvigorw` 调用，按项目配置准备工作区、注入资源、配置 bundle/version/signing、执行构建、复制 HAP 并记录历史。
- 安全：移除可序列化密码字段；macOS 用 Keychain、Windows 用 Credential Manager；缺少发布签名密码时直接阻止 release 构建。

## Test Plan

- 保持 `npm run build`、`cargo check` 通过，新增 `npm run typecheck`。
- Rust 单测：配置迁移、SDK 版本提取、appid 解析、UTS 扫描、模板渲染、XML/plist 修改、图标尺寸、密钥存储 mock。
- 集成测试：用假 SDK/资源渲染 Android 工作区，断言 Gradle/Manifest/strings/control/icons/dependencies 正确；用样例 `Info.plist`/`pbxproj` 验证 iOS 修改；用 fake runner 验证 Harmony 命令。
- UI 冒烟：新建项目、保存配置、导入 `__UNI__`、展示扫描结果、平台可用性 gating、fake 构建日志/产物/历史展示。
- 手动验收：真实 DCloud SDK + HBuilderX 导出资源可生成 Android APK；macOS 可导出 IPA；含 UTS 内置模块和一个自定义插件的样例可自动注入。

## Assumptions

- 权威规格就是 `/Applications/project/tauri/uniapp-pack-tool-plan.md`。
- 修复顺序固定为 Android → iOS → Harmony，因为 Android 是 Phase 1 且会沉淀共享基础设施。
- 兼容旧原型数据只做 best-effort 迁移，不保留会阻断方案落地的旧 schema。
