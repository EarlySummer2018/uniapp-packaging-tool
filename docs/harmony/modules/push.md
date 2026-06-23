# Push 推送服务

> **适用版本**：HBuilderX 5.0+ / uni-app 5.0+
> **平台**：HarmonyOS
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/push.html

---

## 概述

uni-push 统一推送服务，用于在鸿蒙端接收推送消息。

## 配置依赖

统一推送服务依赖此 ohpm 包：

```
@uni_modules/uni-push@1.0.1
```

### 步骤一：添加 ohpm 依赖

在 `oh-package.json5` 文件内 `dependencies` 字段下添加：

```json
{
  "dependencies": {
    "@uni_modules/uni-push": "1.0.1"
  }
}
```

## 配置参数

### 步骤二：配置 module.json5 参数

在项目模块级别下的 `src/main/module.json5` 文件中，新增 metadata 并配置 `GETUI_APPID` 和 `client_id`：

```json
{
  "module": {
    "metadata": [
      {
        "name": "GETUI_APPID",
        "value": "AppID信息"
      },
      {
        "name": "client_id",
        "value": "在华为..."
      }
    ]
  }
}
```

| 参数 | 说明 |
|------|------|
| `GETUI_APPID` | 个推 AppID 信息 |
| `client_id` | 华为推送 client_id |

## 注册模块

### 步骤三：注册 uni-push API

在 uni_modules 入口文件 `index.generated.ets` 内注册 uni-push API：

```typescript
import { uni } from "@dcloudio/uni-app-runtime";
import { getPushClientId, onPushMessage, offPushMessage, createPushMessage, setAppBadgeNumber } from '@uni_modules/uni-push'

export function initUniModules() {
  initUniExtApi();
}

function initUniExtApi() {
  uni.getPushClientId = getPushClientId
  uni.onPushMessage = onPushMessage
  uni.offPushMessage = offPushMessage
  uni.createPushMessage = createPushMessage
  uni.setAppBadgeNumber = setAppBadgeNumber
}
```

## API 说明

| API | 说明 |
|-----|------|
| `getPushClientId` | 获取推送客户端标识 |
| `onPushMessage` | 监听推送消息 |
| `offPushMessage` | 取消监听推送消息 |
| `createPushMessage` | 创建推送消息 |
| `setAppBadgeNumber` | 设置应用角标数字 |

## 相关文档

- [← 返回模块总览](../index.md)
- [通用模块说明](./common.md)
