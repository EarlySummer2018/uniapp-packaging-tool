# 实人认证 (FacialRecognitionVerify)

> **适用版本**：HBuilderX 5.0+ / uni-app 5.0+
> **平台**：HarmonyOS
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/facialRecognitionVerify.html

---

## 概述

实人认证模块，提供人脸识别身份核验能力。

## 配置依赖

实人认证依赖此 ohpm 包：

```
@uni_modules/uni-facialrecognitionverify@1.0.2
```

### 步骤一：添加 ohpm 依赖

在 `oh-package.json5` 文件内 `dependencies` 字段下添加：

```json
{
  "dependencies": {
    "@uni_modules/uni-facialrecognitionverify": "1.0.2"
  }
}
```

## 注册模块

### 步骤二：注册实人认证 API

在 uni_modules 入口文件 `index.generated.ets` 内注册实人认证 API：

```typescript
import { registerUniProvider, uni } from "@dcloudio/uni-app-runtime";
import { startFacialRecognitionVerify, getFacialRecognitionMetaInfo } from '@uni_modules/uni-facialrecognitionverify'

export function initUniModules() {
  initUniExtApi();
}

function initUniExtApi() {
  uni.startFacialRecognitionVerify = startFacialRecognitionVerify
  uni.getFacialRecognitionMetaInfo = getFacialRecognitionMetaInfo
}
```

## API 说明

| API | 说明 |
|-----|------|
| `startFacialRecognitionVerify` | 启动实人认证流程 |
| `getFacialRecognitionMetaInfo` | 获取实人认证所需的元信息（如设备指纹等） |

## 使用方式

配置完成后，可在应用中使用以下方式调用：

```typescript
// 第一步：获取元信息
const metaInfo = await uni.getFacialRecognitionMetaInfo();

// 第二步：将 metaInfo 发送给服务端获取 certifyId
const certifyId = await serverApi.getCertifyId({ metaInfo });

// 第三步：启动实人认证
uni.startFacialRecognitionVerify({
  certifyId: certifyId,
  success: (res) => {
    console.log('实人认证成功', res);
  },
  fail: (err) => {
    console.error('实人认证失败', err);
  }
});
```

## 注意事项

- `certifyId` 必须由服务端生成，不要在客户端伪造
- 认证结果应以服务端的异步通知为准
- 需要获得用户的明确授权才能采集人脸信息
- 建议在真机上测试，模拟器可能不支持

## 相关文档

- [← 返回模块总览](../index.md)
- [通用模块说明](./common.md)
