# FacialRecognitionVerify（实人认证）（iOS）

> **适用版本**：3.7.6+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/facialRecognitionVerify.html
>
> **最后更新**：2025年5月

---

iOS 实人认证模块用于身份验证场景（如金融开户、实名认证等）。

> **注**：实人认证是 `UTS 插件`，依赖 `UTS 基础模块`，集成前需要参考 [UTS 内置模块](uts-builtin-modules.md) 完成 `UTS 基础模块` 的集成。

## 实人认证开发流程

详见 [实人认证开发指南](https://uniapp.dcloud.net.cn/uniCloud/frv/dev.html)

---

## 一、添加依赖库及资源

### 需要引入的第三方库（动态库 / 静态库）

| 类别 | 内容 |
|------|------|
| **动态库** | `DCUniBase.framework`<br>`DCloudUTSFoundation.framework` |
| **静态库/框架** | `uniFacialRecognitionVerify.framework`<br>`AliyunFaceAuthFacade.framework`<br>`AliyunMobileRPC.framework`<br>`AliyunOSSiOS.framework`<br>`APBToygerFacade.framework`<br>`APPSecuritySDK.framework`<br>`BioAuthAPI.framework`<br>`BioAuthEngine.framework`<br>`deviceiOS.framework`<br>`DTFIdentityManager.framework`<br>`DTFSensorServices.framework`<br>`DTFUIModule.framework`<br>`DTFUtility.framework`<br>`MPRemoteLogging.framework`<br>`ToygerNative.framework`<br>`ToygerService.framework` |

### 需要引入的系统库

| 系统库 | 说明 |
|--------|------|
| `CoreGraphics.framework` | 图形绘制 |
| `Accelerate.framework` | 加速框架 |
| `SystemConfiguration.framework` | 系统网络配置 |
| `AssetsLibrary.framework` | 资产库访问 |
| `CoreTelephony.framework` | 核心电话框架 |
| `QuartzCore.framework` | 核心动画 |
| `CoreFoundation.framework` | 基础核心框架 |
| `CoreLocation.framework` | 定位服务 |
| `ImageIO.framework` | 图像 I/O |
| `CoreMedia.framework` | 核心媒体 |
| `CoreMotion.framework` | 运动传感器 |
| `AVFoundation.framework` | 音视频采集（人脸检测必需） |
| `WebKit.framework` | WebKit 引擎 |
| `AudioToolbox.framework` | 音频工具箱 |
| `CFNetwork.framework` | 网络通信 |
| `MobileCoreServices.framework` | 移动核心服务 |
| `AdSupport.framework` | 广告支持（设备标识） |
| `libresolv.tbd` | DNS 解析库 |
| `libz.tbd` | zlib 压缩库 |
| `libc++.tbd` | C++ 运行时 |
| `libc++.1.tbd` | C++ 运行时（指定版本） |
| `libc++abi.tbd` | C++ ABI 库 |
| `libz.1.2.8.tbd` | zlib（指定版本） |

### 需要拷贝的资源文件（Bundle）

| 路径 | 文件 |
|------|------|
| SDK/resources | `APBToygerFacade.bundle` |
| SDK/resources | `BioAuthEngine.bundle` |
| SDK/resources | `ToygerNative.bundle` |

---

## 二、隐私权限配置（Info.plist）

在 Info.plist 文件中添加相机权限描述：

```xml
<!-- 相机权限（人脸识别必需） -->
<key>NSCameraUsageDescription</key>
<string>我们需要使用摄像头进行人脸识别验证</string>
```

---

## ⚠️ 重要注意事项

1. **企业实名备案**：使用实人认证功能需要进行企业实名认证
2. **安全合规**：人脸数据属于敏感信息，需符合《个人信息保护法》要求
3. **活体检测**：SDK 内置活体检测功能，可有效防止照片/视频攻击
4. **网络环境**：认证过程需要联网，且对网络质量有要求（依赖阿里云服务）
5. **真机测试**：模拟器不支持相机调用，必须使用真机测试
6. **UTS 基础模块前置依赖**：集成前务必先完成 [UTS 内置模块](uts-builtin-modules.md) 的集成

---

## 交叉引用

- 上一篇：[Statistic（统计）](statistic.md)
- 下一篇：[uni-AD（广告）](uni-ad.md)
- 相关模块：[LivePusher（直播推流）](livepusher.md)（同样需要相机权限）
- 前置依赖：[UTS 内置模块](uts-builtin-modules.md)
