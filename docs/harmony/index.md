# 鸿蒙 HarmonyOS 模块配置概览

> **适用版本**：HBuilderX 5.0+
> **平台**：HarmonyOS Next (API 12+)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/
> **整理时间**：2026-05-29

---

本文档为 DCloud UniApp 鸿蒙（HarmonyOS）离线 SDK 的模块配置总览页，提供各模块的一站式导航与快速参考。

## 📋 当前支持模块一览表

| 序号 | 模块名称 | 功能描述 | ohpm 包名 | 版本 | 注册方式 | 配置复杂度 | 详细文档 |
|------|---------|---------|-----------|------|---------|-----------|---------|
| 1 | **OAuth** | 华为登录 | `@uni_modules/uni-oauth-huawei` | 1.0.1 | `registerUniProvider` | ⭐⭐ | [→ 查看详情](./modules/oauth.md) |
| 2 | **Map** | 腾讯地图 | 内置模块 | - | metadata 配置 | ⭐ | [→ 查看详情](./modules/map.md) |
| 3 | **Payment** | 支付宝支付 | `@uni_modules/uni-payment-alipay` | 1.0.1 | `registerUniProvider` | ⭐⭐⭐ | [→ 查看详情](./modules/payment.md) |
| 4 | **FaceRecognition** | 实人认证 | `@uni_modules/uni-facialrecognitionverify` | 1.0.2 | uni 全局方法 | ⭐⭐⭐ | [→ 查看详情](./modules/facial-recognition-verify.md) |

> 💡 **说明**：鸿蒙端目前支持的模块相对较少，后续会持续增加。暂未支持的常用模块包括：微信登录/支付、推送（Push）、统计（Statistic）、分享等。

---

## 📂 各模块快速链接

- [🔐 OAuth 登录鉴权（华为登录）](./modules/oauth.md) — 集成华为账号登录能力
- [🗺️ Map 地图（腾讯地图）](./modules/map.md) — 集成腾讯地图显示和交互能力
- 💳 Payment 支付（支付宝）](./modules/payment.md) — 集成支付宝支付能力
- 👤 FacialRecognitionVerify 实人认证](./modules/facial-recognition-verify.md) — 提供实人认证（人脸识别）能力

---

## 📝 配置文件修改清单

| 配置文件 | 修改内容 | 涉及模块 |
|---------|---------|---------|
| `oh-package.json5` | 添加 dependencies 依赖 | OAuth, Payment, FaceRecognition |
| `index.generated.ets` | 注册 Provider 或扩展 API | OAuth, Payment, FaceRecognition |
| `module.json5` | 添加 metadata 配置 | Map |

---

## 🖥️ 开发环境要求

| 环境 | 要求 |
|------|------|
| **IDE** | DevEco Studio 4.0+ |
| **SDK** | HarmonyOS SDK API 12+ |
| **包管理** | ohpm（鸿蒙包管理工具） |
| **编程语言** | ArkTS（TypeScript 扩展） |
| **运行时** | uni-app 鸿蒙运行时（HBuilderX 5.0+） |
| **签名** | 鸿蒙应用签名文件（发布必需） |

---

## ⚡ 快速开始

如果你需要同时配置所有模块，可参考以下模板：

**oh-package.json5**:
```json
{
  "dependencies": {
    "@uni_modules/uni-oauth-huawei": "1.0.1",
    "@uni_modules/uni-payment-alipay": "1.0.1",
    "@uni_modules/uni-facialrecognitionverify": "1.0.2"
  }
}
```

**index.generated.ets**:
```typescript
import { registerUniProvider, uni } from "@dcloudio/uni-app-runtime";
import { UniOAuthHuaweiProviderImpl } from "@uni_modules/uni-oauth-huawei";
import { UniPaymentAlipayProviderImpl } from "@uni_modules/uni-payment-alipay";
import { 
  startFacialRecognitionVerify, 
  getFacialRecognitionMetaInfo 
} from '@uni_modules/uni-facialrecognitionverify';

export function initUniModules() {
  initUniExtApi();
}

function initUniExtApi() {
  // 注册华为登录
  registerUniProvider("oauth", "huawei", new UniOAuthHuaweiProviderImpl());
  
  // 注册支付宝支付
  registerUniProvider("payment", "alipay", new UniPaymentAlipayProviderImpl());
  
  // 注册实人认证 API
  uni.startFacialRecognitionVerify = startFacialRecognitionVerify;
  uni.getFacialRecognitionMetaInfo = getFacialRecognitionMetaInfo;
}
```

**module.json5** (仅地图):
```json
{
  "module": {
    "metadata": [
      {
        "name": "TENCENT_MAP_KEY",
        "value": "你的腾讯地图Key"
      }
    ]
  }
}
```

---

## 📚 更多资源

- [❓ 常见问题 FAQ](./faq.md) — 配置过程中的常见问题排查
- [🔄 与其他平台对比](./faq.md#与其他平台对比) — Android/iOS/鸿蒙配置差异对照
- [⚠️ 重要提示](./faq.md#重要提示) — 关键注意事项汇总

---

## 📎 相关文档

- **DCloud 鸿蒙离线 SDK 文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/
- **HarmonyOS 开发指南**：https://developer.huawei.com/consumer/cn/harmonyos/doc/
- **ohpm 使用文档**：https://developer.huawei.com/consumer/cn/doc/harmonyos-guides/
- **uni-app 官方文档**：https://uniapp.dcloud.net.cn/
