# FacialRecognitionVerify 实人认证（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 9. FacialRecognitionVerify（实人认证）

> **适用版本**：HBuilderX 5.0+
> 
> **前置依赖**：实人认证依赖于 UTS 基础模块，请先集成 [UTS 基础模块](uts-base-module.md)

### 需要拷贝的文件

**需要引入工程的 aar 文件**（放到工程的 `libs` 目录下）：

| 路径 | 文件列表 |
|---|---|
| **SDK\libs** | `uni-facialVerify-release.aar`<br>`aliyun-base-XXX.aar`<br>`aliyun-facade-XXX.aar`<br>`aliyun-face-XXX.aar`<br>`aliyun-faceaudio-XXX.aar`<br>`aliyun-facelanguage-XXX.aar`<br>`aliyun-photinus-XXX.aar`<br>`aliyun-wishverify-XXX.aar`<br>`APSecuritySDK-DeepSec-*.jiagu.aar`<br>`Android-AliyunFaceGuard-10042.aar`<br>`APSecuritySDK-DeepSec-7.0.1.20230914.jiagu.aar`<br>`facialRecognitionVerify-support-release.aar` |

> **注意**：`XXX` 为版本号，具体版本号以下载的 SDK 中的为准
>
> **X86 设备支持说明**：HBuilderX 新增了 `facialRecognitionVerify-support-release.aar` 库，作用是应用可以在 X86 设备上正常运行，但调用 `uni.startFacialRecognitionVerify()` 会触发错误回调。如果不支持 X86 设备，可以不用引入。

### app 级 build.gradle 配置

```groovy
dependencies {
    implementation "com.squareup.okhttp3:okhttp:3.11.0"
    implementation "com.squareup.okio:okio:1.14.0"
    implementation "Com.aliyun.dpa:oss-android-sdk:+"
}
```

### 错误处理配置

**问题**：离线 SDK 集成实人认证如果出现 `lib/*/libc++_shared.so` 报错时

**解决方案**：需要在 module 的 `build.gradle` 的 android 节点下添加如下内容：

```groovy
android {
    packagingOptions {
        pickFirst 'lib/*/libc++_shared.so'
    }
}
```

### API 调用示例

实人认证功能通过 uni-app API 调用：

```javascript
// 启动实人认证
uni.startFacialRecognitionVerify({
    success: (res) => {
        console.log('认证成功', res)
    },
    fail: (err) => {
        console.log('认证失败', err)
    }
})

// 获取实人认证 MetaInfo（用于服务端校验）
const metaInfo = uni.getFacialRecognitionMetaInfo()
console.log('MetaInfo:', metaInfo)
```

---

### 相关模块

- [UTS 基础模块](uts-base-module.md) — 实人认证的前置依赖模块
- [FAQ](../faq.md) — FAQ第5条包含实人认证libc++_shared.so报错解决方案
