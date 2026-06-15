# DCloud 原生模块配置指南

> 本文档汇总了 Android、iOS、鸿蒙平台下常用原生模块的配置文档，已按平台和模块拆分为独立文件。开发者可根据需要选择对应平台和模块进行集成配置。

## 平台导航

### Android

> **适用版本**：HBuilderX 5.0+
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

| 分类 | 模块 | 文档 |
|------|------|------|
| **工程配置** | SDK 集成指南 | [android/index.md](android/index.md) |
| **启动图** | 启动图配置 | [android/launch-config.md](android/launch-config.md) |
| **基础能力** | 定位（百度/高德/系统/腾讯） | [android/modules/geolocation.md](android/modules/geolocation.md) |
| | 语音输入（百度/讯飞） | [android/modules/speech.md](android/modules/speech.md) |
| | X5 WebView（腾讯 TBS 内核） | [android/modules/x5-webview.md](android/modules/x5-webview.md) |
| | UTS 内置模块 | [android/modules/uts-builtin-modules.md](android/modules/uts-builtin-modules.md) |
| | UTS 基础模块 | [android/modules/uts-base-module.md](android/modules/uts-base-module.md) |
| | 其他模块（视频/直播/扫码等） | [android/modules/other-modules.md](android/modules/other-modules.md) |
| **社交与分享** | 分享（微信/QQ/微博） | [android/modules/share.md](android/modules/share.md) |
| | 登录鉴权（7种登录方式） | [android/modules/oauth.md](android/modules/oauth.md) |
| **地图与位置** | 地图（百度/高德/谷歌） | [android/modules/map.md](android/modules/map.md) |
| **支付与推送** | 支付（支付宝/微信/PayPal等） | [android/modules/payment.md](android/modules/payment.md) |
| | 消息推送 / uniPush | [android/modules/push.md](android/modules/push.md) |
| **广告与统计** | 广告（穿山甲/优量汇/快手等） | [android/modules/uni-ad.md](android/modules/uni-ad.md) |
| | 统计（友盟/谷歌） | [android/modules/statistic.md](android/modules/statistic.md) |
| **安全认证** | 实人认证 | [android/modules/facial-recognition-verify.md](android/modules/facial-recognition-verify.md) |
| **依赖说明** | 第三方 SDK 依赖说明 | [android/modules/third-party-dependencies.md](android/modules/third-party-dependencies.md) |
| **FAQ** | Android 注意事项（26条） | [android/faq.md](android/faq.md) |

---

### iOS

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

| 分类 | 模块 | 文档 |
|------|------|------|
| **消息推送** | Push / uniPush | [ios/modules/push.md](ios/modules/push.md) |
| **社交分享** | 分享（微信/QQ/微博/Facebook） | [ios/modules/share.md](ios/modules/share.md) |
| **登录鉴权** | Oauth（7种登录含Apple登录） | [ios/modules/oauth.md](ios/modules/oauth.md) |
| **地图** | 地图（百度/高德/苹果原生） | [ios/modules/map.md](ios/modules/map.md) |
| **语音输入** | Speech（百度/讯飞） | [ios/modules/speech.md](ios/modules/speech.md) |
| **直播推流** | LivePusher（又拍云直播推流） | [ios/modules/livepusher.md](ios/modules/livepusher.md) |
| **统计** | Statistic（友盟/谷歌分析） | [ios/modules/statistic.md](ios/modules/statistic.md) |
| **实人认证** | FacialRecognitionVerify | [ios/modules/facial-recognition-verify.md](ios/modules/facial-recognition-verify.md) |
| **广告** | uni-AD（穿山甲/优量汇/快手等） | [ios/modules/uni-ad.md](ios/modules/uni-ad.md) |
| **WebView** | UIWebview 配置 | [ios/modules/uiwebview.md](ios/modules/uiwebview.md) |
| **UTS模块** | UTS 内置模块 | [ios/modules/uts-builtin-modules.md](ios/modules/uts-builtin-modules.md) |
| **定位** | Geolocation（百度/高德/系统） | [ios/modules/geolocation.md](ios/modules/geolocation.md) |
| **支付** | Payment（支付宝/微信/IAP/ApplePay等） | [ios/modules/payment.md](ios/modules/payment.md) |
| **依赖说明** | 第三方 SDK 依赖 + Podfile 示例 | [ios/modules/third-party-dependencies.md](ios/modules/third-party-dependencies.md) |
| **FAQ** | iOS 注意事项（10条） | [ios/faq.md](ios/faq.md) |

---

### 鸿蒙 HarmonyOS

> **适用版本**：HBuilderX 5.0+
> **平台**：HarmonyOS Next (API 12+)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/

| 模块 | 功能 | 文档 |
|------|------|------|
| OAuth | 华为登录 | [harmony/modules/oauth.md](harmony/modules/oauth.md) |
| Map | 腾讯地图（Web方案） | [harmony/modules/map.md](harmony/modules/map.md) |
| Payment | 支付宝支付 | [harmony/modules/payment.md](harmony/modules/payment.md) |
| FacialRecognitionVerify | 实人认证 | [harmony/modules/facial-recognition-verify.md](harmony/modules/facial-recognition-verify.md) |
| **概览** | 鸿蒙总览+快速配置模板 | [harmony/index.md](harmony/index.md) |
| **FAQ** | 常见问题 + 平台对比 | [harmony/faq.md](harmony/faq.md) |

> 注：鸿蒙平台目前支持以上 4 个模块，更多模块持续适配中。

---

## 目录结构总览

```
docs/
├── overview.md                    # 本文件 - 总览索引页
├── android/                      # Android 平台（18个文件）
│   ├── index.md                  # 工程配置指南
│   ├── launch-config.md          # 启动图配置
│   ├── faq.md                    # 注意事项（26条）
│   └── modules/                  # 15个模块文档
├── ios/                          # iOS 平台（15个文件）
│   ├── faq.md                    # 注意事项（10条）
│   └── modules/                  # 14个模块文档
└── harmony/                      # 鸿蒙平台（6个文件）
    ├── index.md                  # 鸿蒙概览
    ├── faq.md                    # FAQ + 平台对比
    └── modules/                  # 4个模块文档
```

---

## 快速开始

1. 选择目标平台目录（`android/`、`ios/` 或 `harmony/`）
2. 阅读 `index.md` 了解工程配置要求
3. 根据需要集成的模块，进入 `modules/` 目录查看对应文档
4. 遇到问题时查看各平台的 `faq.md`
