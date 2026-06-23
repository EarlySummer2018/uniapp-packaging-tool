# 模块配置通用说明

> **适用版本**：HBuilderX 5.0+ / uni-app 5.0+
> **平台**：HarmonyOS
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/common.html

---

## 概述

本文档提供鸿蒙模块配置的通用说明和注意事项。

## 涉及配置的模块

以下模块需要配置依赖或参数：

| 模块 | ohpm 包名 | 版本 | 配置方式 |
|------|-----------|------|----------|
| Map | 内置模块 | - | metadata 配置 |
| Push | `@uni_modules/uni-push` | 1.0.1 | 依赖 + 参数 + 注册 |
| OAuth | `@uni_modules/uni-oauth-huawei` | 1.0.1 | 依赖 + 注册 |
| FacialRecognitionVerify | `@uni_modules/uni-facialrecognitionverify` | 1.0.2 | 依赖 + 注册 |
| Payment | `@uni_modules/uni-payment-alipay` | 1.0.1 | 依赖 + 注册 |

## 自动配置 vs 手动配置

### 自动配置（推荐）

如在 uni-app 项目 `manifest.json` 内已勾选对应的鸿蒙模块：

1. 编译产物的 `uni_modules` 目录下会自动生成：
   - `index.generated.ets`
   - `oh-package.json5`

2. 参考[集成编译产物到项目内](../integration-guide.md)文档集成即可

### 手动配置

如需手动配置模块，请参考各模块的详细文档。

## 关键文件说明

### index.generated.ets

- **路径**：`/entry/src/main/ets/uni_modules/index.generated.ets`
- **作用**：鸿蒙原生工程内的 uni_modules 入口文件
- **内容**：模块注册代码，包括 Provider 注册和 API 扩展

### oh-package.json5

- **路径**：根目录 `/oh-package.json5`
- **作用**：声明 ohpm 依赖包
- **格式**：在 `dependencies` 字段中添加依赖

```json
{
  "dependencies": {
    "@uni_modules/xxx": "版本号"
  }
}
```

### module.json5

- **路径**：`/entry/src/main/module.json5`
- **作用**：模块元数据配置
- **用途**：配置 API Key、AppID 等参数

```json
{
  "module": {
    "metadata": [
      {
        "name": "参数名称",
        "value": "参数值"
      }
    ]
  }
}
```

## build-profile.json5 兼容性配置

> **重要**：需要在工程级的 `build-profile.json5` 的 `products` 字段中配置 `compatibleSdkVersionStage: "beta6"`。

## 模块注册方式

### 方式一：registerUniProvider（Provider 类型）

适用于 OAuth、Payment 等模块：

```typescript
import { registerUniProvider } from "@dcloudio/uni-app-runtime";
import { XxxProviderImpl } from "@uni_modules/xxx";

export function initUniModules() {
  initUniExtApi();
}

function initUniExtApi() {
  registerUniProvider("模块类型", "服务商标识", new XxxProviderImpl());
}
```

### 方式二：uni 全局方法扩展（API 类型）

适用于 Push、FacialRecognitionVerify 等模块：

```typescript
import { uni } from "@dcloudio/uni-app-runtime";
import { xxxMethod, yyyMethod } from '@uni_modules/xxx';

export function initUniModules() {
  initUniExtApi();
}

function initUniExtApi() {
  uni.xxxMethod = xxxMethod;
  uni.yyyMethod = yyyMethod;
}
```

## 相关文档

- [← 返回模块总览](../index.md)
- [腾讯地图 (Map)](./map.md)
- [推送服务 (Push)](./push.md)
- [华为登录 (OAuth)](./oauth.md)
- [实人认证 (FacialRecognitionVerify)](./facial-recognition-verify.md)
- [支付宝支付 (Payment)](./payment.md)
