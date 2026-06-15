# UTS 基础模块 & UTS 内置模块（iOS）

> **适用版本**：3.7.6+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/uts.html
>
> **最后更新**：2025年5月

---

使用 `UTS 插件`、`实人认证模块` 以及 `UTS 内置模块` 必须先集成 `UTS 基础模块`。

---

## 一、UTS 基础模块

### 需要添加的依赖库（主工程）

以下两个框架必须添加到主工程，并设置为 **Embed & Sign**：

| 框架 | 类型 | 说明 |
|------|------|------|
| `DCUniBase.framework` | 动态库（Embed & Sign） | DCloud UniApp 基础框架，包含核心运行时 |
| `DCloudUTSFoundation.framework` | 动态库（Embed & Sign） | UTS 运行时基础框架 |

### 需要移除的依赖库（避免重复引用）

> **重要**：`DCUniBase.framework` 内部已包含以下依赖库，主工程中**必须移除**，否则会出现重复符号（duplicate symbols）错误：

| 库/框架 | 说明 |
|--------|------|
| `liblibPDRCore.a` | PDR 核心静态库 |
| `liblibWeex.a` | Weex 引擎静态库 |
| `libcoreSupport.a` | 核心支持库 |
| `storage.framework` | 存储框架 |
| `libSDWebImage.a` | SDWebImage 图片缓存库 |
| `KSCrash.framework` | 崩溃收集框架 |

---

## 二、UTS 内置模块

### 模块列表

UTS 内置模块提供了一系列常用的 uni API 原生实现，包含以下模块：

| 模块名称 | 功能说明 |
|---------|---------|
| `uni-chooseMedia` | 选择媒体文件（图片/视频） |
| `uni-getAppAuthorizeSetting` | 获取应用授权设置状态 |
| `uni-getAppBaseInfo` | 获取应用基础信息 |
| `uni-getDeviceInfo` | 获取设备信息 |
| `uni-getLocation-tencent-uni1` | 获取地理位置（腾讯定位） |
| `uni-getNetworkType` | 获取网络类型 |
| `uni-getSystemInfo` | 获取系统信息 |
| `uni-getSystemSetting` | 获取系统设置 |
| `uni-network` | 网络请求能力 |
| `uni-openAppAuthorizeSetting` | 打开应用授权设置页面 |
| `uni-privacy` | 隐私相关 API |
| `uni-prompt` | 弹窗提示 |
| `uni-storage` | 本地数据存储 |

### 集成方式

如需使用上述内置模块，在主工程中添加以下框架即可：

| 框架 | 类型 | 说明 |
|------|------|------|
| `DCloudUTSExtAPI.framework` | 动态库（Embed & Sign） | UTS 扩展 API 框架，包含所有内置模块的实现 |

### 使用场景说明

| 场景 | 是否需要添加 ext-api |
|------|---------------------|
| 在 uni-app 项目中直接调用 `uni.getDeviceInfo()` 等 API | **不需要** |
| 在 **UTS 插件** 中调用 `uni.getDeviceInfo()` 等 API | **需要** 添加 `DCloudUTSExtAPI.framework` |

> **注**：在普通 uni-app 页面中使用 uni API 无需额外依赖；但在 UTS 插件内部调用 uni API 时，必须引入 `DCloudUTSExtAPI.framework`。

---

## ⚠️ 重要注意事项

1. **依赖顺序**：务必先完成 UTS 基础模块集成（DCUniBase + DCloudUTSFoundation），再集成其他依赖 UTS 的模块（如实人认证）
2. **重复引用检查**：移除 DCUniBase 内已包含的 6 个库/框架是**必须的步骤**，遗漏会导致编译失败
3. **Embed & Sign 设置**：DCUniBase、DCloudUTSFoundation、DCloudUTSExtAPI 三个动态库都必须设置为 Embed & Sign，否则运行时会找不到符号
4. **UTS 插件开发**：若在自定义 UTS 插件中使用了 uni API（如 `uni.getDeviceInfo()`），请确保已添加 DCloudUTSExtAPI.framework

---

## 交叉引用

- 上一篇：[UIWebview 配置](uiwebview.md)
- 下一篇：[Geolocation（定位）](geolocation.md)
- 相关模块：[FacialRecognitionVerify（实人认证）](facial-recognition-verify.md)（依赖本模块）、[Push（推送）](push.md)
