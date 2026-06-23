# OAuth 登录鉴权

> **适用版本**：HBuilderX 5.0+ / uni-app 5.0+
> **平台**：HarmonyOS
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/oauth.html

---

## 概述

OAuth 模块提供了第三方登录的能力，目前仅支持华为登录。

## 配置依赖

华为登录依赖此 ohpm 包：

```
@uni_modules/uni-oauth-huawei@1.0.1
```

### 步骤一：添加 ohpm 依赖

在 `oh-package.json5` 文件内 `dependencies` 字段下添加：

```json
{
  "dependencies": {
    "@uni_modules/uni-oauth-huawei": "1.0.1"
  }
}
```

## 注册模块

### 步骤二：注册华为登录 Provider

在 uni_modules 入口文件 `index.generated.ets` 内注册华为登录 provider：

```typescript
import { registerUniProvider, uni } from "@dcloudio/uni-app-runtime";
import { UniOAuthHuaweiProviderImpl } from "@uni_modules/uni-oauth-huawei";

export function initUniModules() {
  initUniExtApi();
}

function initUniExtApi() {
  registerUniProvider("oauth", "huawei", new UniOAuthHuaweiProviderImpl());
}
```

## 代码说明

| 参数 | 说明 |
|------|------|
| `"oauth"` | 模块类型标识，表示这是登录鉴权模块 |
| `"huawei"` | 服务提供商标识，表示使用华为登录 |
| `new UniOAuthHuaweiProviderImpl()` | 创建华为登录 Provider 实例 |

## 使用方式

配置完成后，可在应用中使用以下方式调用：

```typescript
uni.login({
  provider: 'huawei',
  success: (res) => {
    console.log('华为登录成功', res);
  },
  fail: (err) => {
    console.error('华为登录失败', err);
  }
});
```

## 相关文档

- [← 返回模块总览](../index.md)
- [通用模块说明](./common.md)
