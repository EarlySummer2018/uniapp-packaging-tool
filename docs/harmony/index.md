# 鸿蒙模块配置总览

> **适用版本**：HBuilderX 5.0+ / uni-app 5.0+
> **平台**：HarmonyOS
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/

---

## 概述

模块配置文档适用于 HBuilderX 5.0+ / uni-app 5.0+。

目前如下模块涉及配置依赖或参数：

| 模块 | 功能 | 详细文档 |
|------|------|----------|
| [Map](./modules/map.md) | 腾讯地图 | [→ 查看详情](./modules/map.md) |
| [Push](./modules/push.md) | 统一推送服务 | [→ 查看详情](./modules/push.md) |
| [OAuth](./modules/oauth.md) | 华为登录 | [→ 查看详情](./modules/oauth.md) |
| [FacialRecognitionVerify](./modules/facial-recognition-verify.md) | 实人认证 | [→ 查看详情](./modules/facial-recognition-verify.md) |
| [Payment](./modules/payment.md) | 支付宝支付 | [→ 查看详情](./modules/payment.md) |

## 自动配置说明

如在 uni-app 项目 `manifest.json` 内已勾选对应的鸿蒙模块，则在编译产物的 `uni_modules` 目录下会生成对应的：

- `index.generated.ets` — 模块入口文件
- `oh-package.json5` — 依赖配置文件

参考[集成编译产物到项目内](./integration-guide.md)文档将这两个文件集成到鸿蒙项目内即可。

如需手动配置模块，请参考各模块的详细文档。

## 关键文件路径约定

为简化描述，本文档约定以下概念：

| 文件 | 路径 | 说明 |
|------|------|------|
| `index.generated.ets` | `/entry/src/main/ets/uni_modules/index.generated.ets` | 鸿蒙原生工程内的 uni_modules 入口文件 |
| `oh-package.json5` | 根目录 `/oh-package.json5` | 鸿蒙原生工程内的包管理配置文件 |
| `module.json5` | `/entry/src/main/module.json5` | 模块配置文件（用于 metadata 配置） |
| `build-profile.json5` | 根目录 `/build-profile.json5` | 工程级构建配置文件 |

## HBuilderX 4.51+ 重要配置

> **注意**：HBuilder X 升级至 4.51 后，需要在工程级的 `build-profile.json5` 的 `products` 字段中配置 `compatibleSdkVersionStage: "beta6"`。

如果有多项 products，都需要配置：

```json
{
  "app": {
    "products": [
      {
        "name": "default",
        "compatibleSdkVersionStage": "beta6",
        // ... 其他配置
      }
    ]
  }
}
```

## 快速导航

### 项目配置

- [配置鸿蒙原生项目](./setup-guide.md) — 初始项目配置
- [集成编译产物到项目内](./integration-guide.md) — 编译产物集成步骤

### 模块配置

- [通用模块说明](./modules/common.md) — 模块配置通用指南
- [腾讯地图 (Map)](./modules/map.md)
- [推送服务 (Push)](./modules/push.md)
- [华为登录 (OAuth)](./modules/oauth.md)
- [实人认证 (FacialRecognitionVerify)](./modules/facial-recognition-verify.md)
- [支付宝支付 (Payment)](./modules/payment.md)

## 相关文档

- **DCloud 官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/
- **uni-app 官方文档**：https://uniapp.dcloud.net.cn/
