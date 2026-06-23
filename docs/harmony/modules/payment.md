# 支付宝支付 (Payment)

> **适用版本**：HBuilderX 5.0+ / uni-app 5.0+
> **平台**：HarmonyOS
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/pay.html

---

## 概述

支付宝支付模块，用于在鸿蒙端集成支付宝支付能力。

## 配置依赖

支付宝支付依赖此 ohpm 包：

```
@uni_modules/uni-payment-alipay@1.0.1
```

### 步骤一：添加 ohpm 依赖

在 `oh-package.json5` 文件内 `dependencies` 字段下添加：

```json
{
  "dependencies": {
    "@uni_modules/uni-payment-alipay": "1.0.1"
  }
}
```

## 注册模块

### 步骤二：注册支付宝支付 Provider

在 uni_modules 入口文件 `index.generated.ets` 内注册支付宝支付 provider：

```typescript
import { registerUniProvider, uni } from "@dcloudio/uni-app-runtime";
import { UniPaymentAlipayProviderImpl } from "@uni_modules/uni-payment-alipay";

export function initUniModules() {
  initUniExtApi();
}

function initUniExtApi() {
  registerUniProvider("payment", "alipay", new UniPaymentAlipayProviderImpl());
}
```

## 代码说明

| 参数 | 说明 |
|------|------|
| `"payment"` | 模块类型标识，表示这是支付模块 |
| `"alipay"` | 服务提供商标识，表示使用支付宝支付 |
| `new UniPaymentAlipayProviderImpl()` | 创建支付宝支付 Provider 实例 |

## 使用方式

配置完成后，可在应用中使用以下方式调用：

```typescript
uni.requestPayment({
  provider: 'alipay',
  orderInfo: '从服务端获取的支付参数',
  success: function (res) {
    console.log('支付成功', res);
    // 注意：最终支付状态应以服务端异步通知为准
  },
  fail: function (err) {
    console.log('支付失败', err);
  }
});
```

## 注意事项

- `orderInfo` 参数必须由服务端生成，不要在客户端拼接
- 不要在客户端存储支付宝私钥等敏感信息
- 支付结果应以服务端异步通知为准，不能仅依赖客户端返回值
- 发布版应用必须使用正式签名，确保与支付宝开放平台配置一致

## 相关文档

- [← 返回模块总览](../index.md)
- [通用模块说明](./common.md)
