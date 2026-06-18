# Statistic（统计）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/statistic.html

---

## HBuilderX 5.13+ 本地 Pod 集成（推荐）

HBuilderX 5.13+ 推荐使用本地 Pod 集成统计模块。统计基础模块使用 `Statistic`，友盟统计使用 `Statistic-Umeng`，Firebase 统计使用 `Statistic-Firebase`。

手动集成时再参考下方依赖表；`Statistic-Firebase` 通常还需要添加 Firebase 生成的 `GoogleService-Info.plist`。

## 友盟统计

### 将友盟统计模块依赖库及资源添加到工程

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibStatistic.a、libUmengStatistic.a、UMDevice.xcframework、UMCommon.xcframework、UMAPM.framework | libz.tbd、libsqlite3.tbd、SystemConfiguration.framework、CoreTelephony.framework | 无 |

### 帐号配置

1. 到[友盟开放平台](http://www.umeng.com/analytics)申请 Appkey。

2. 打开 Info.plist 文件找到 `umeng` 项，如果没有按图片中的格式添加该项，在下图中的红色区域输入申请的 Appkey。

**注意：** IDFA 说明

从 HBuilderX 2.2.5 版本之后（含 2.2.5），基座里集成了友盟 v6.0.5 统计 SDK，因友盟官方，从组件化产品开始，【友盟+】SDK 默认采集 idfa 标识，用来更准确的分析核对数据。对于应用本身没有获取 idfa 的情况，建议将应用提交至 AppStore 时按如下方式配置：（以避免被苹果以"应用不含广告功能，但获取了广告标示符 IDFA"的而拒绝其上架。）

### 隐私清单

## Firebase Analytics （SDK 3.3.7+ 新增）

### 将 Firebase Analytics 模块依赖库及资源添加到工程

需要在 `/SDK/Bundles/PandoraApi.bundle/feature.plist` 文件中修改如下字段：

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| liblibStatistic.a、libGoogleStatistic.a、FirebaseCore.xcframework、FirebaseCoreInternal.xcframework、FirebaseInstallations.xcframework、GoogleAppMeasurement.xcframework、GoogleAppMeasurementIdentitySupport.xcframework、GoogleUtilities.xcframework、FBLPromises.xcframework、nanopb.xcframework | 无 | GoogleService-Info.plist |

### 帐号配置

1. 在 [Firebase 官网](https://firebase.google.com/)创建新项目或找到已创建项目。
2. 下载 Firebase 生成的 `GoogleService-Info.plist` 加到工程中。

---

## 交叉引用

- 上一篇：[LivePusher（直播推流）](livepusher.md)
- 下一篇：[FacialRecognitionVerify（实人认证）](facial-recognition-verify.md)
- 相关模块：[uni-AD（广告）](uni-ad.md)（同样涉及隐私合规）、[Push（消息推送）](push.md)（可结合推送+统计）
