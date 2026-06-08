# uni-AD（广告）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

iOS 广告模块支持穿山甲、优量汇、快手、Sigmob、百度等多个广告平台。

> **配置前提**：需先在 [DCloud 广告联盟](https://uniad.dcloud.net.cn) 申请账号并开通相应广告位。

## 公共配置

### Info.plist 配置

```xml
<!-- 广告基础配置 -->
<key>DCLOUD_AD_SPLASH</key>
<true/>  <!-- 是否开启开屏广告 -->

<key>DCLOUD_STREAMAPP_CHANNEL</key>
<string>%包名|%appid|%广告标识|%渠道%</string>
<!-- 示例：com.example.app|1234567890|AD10001|AppStore -->
```

字段说明：
- **包名**：应用的 Bundle Identifier
- **应用标识**：manifest.json 中的 appid
- **广告标识**：联盟ID，可在 uniad.dcloud.net.cn 获取
- **渠道**：分发渠道名称（如 App Store、TestFlight 等）

## 9.1 穿山甲（字节跳动广告）

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| AdSupport.framework | 广告标识符 |
| StoreKit.framework | 应用内购买（激励视频） |
| MobileCoreServices.framework | 移动核心服务 |
| WebKit.framework | 网页渲染 |
| CoreMedia.framework | 核心媒体库 |
| CoreLocation.framework | 定位（精准投放） |
| CoreTelephony.framework | 设备信息 |
| SystemConfiguration.framework | 网络状态 |
| libz.tbd | 数据压缩 |
| libc++.tbd | C++ 运行时 |
| libsqlite3.tbd | 本地存储 |

### CocoaPods 依赖

```ruby
pod 'Bytedance-UnionSDK', '~> 5.x.x'  # 穿山甲广告SDK
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `BUAdSDK.framework`, `CSJMTGRewardVideoAdapter.framework` 等 |

### dcloud_properties.xml 配置

```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="csj" value="io.dcloud.feature.ad.csj.ADCsjModule"/>
</feature>
```

## 9.2 腾讯优量汇（GDT）

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| AdSupport.framework | 广告标识符 |
| StoreKit.framework | 应用内购买 |
| CoreTelephony.framework | 设备信息 |
| CoreGraphics.framework | 图形渲染 |
| QuartzCore.framework | 动画效果 |
| CoreLocation.framework | 定位 |
| WebKit.framework | 网页渲染 |
| libz.tbd | 压缩库 |

### CocoaPods 依赖

```ruby
pod 'GDTMobSDK', '~> 4.x.x'  # 优量汇SDK
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `GDTMobSDK.framework` |

### dcloud_properties.xml 配置

```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="gdt" value="io.dcloud.feature.ad.gdt.ADGdtModule"/>
</feature>
```

## 9.3 快手广告

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| AdSupport.framework | 广告标识符 |
| CoreLocation.framework | 定位 |
| CoreTelephony.framework | 设备信息 |
| SystemConfiguration.framework | 网络状态 |
| Security.framework | 安全服务 |
| libz.tbd | 压缩库 |
| libc++.tbd | C++ 运行时 |

### CocoaPods 依赖

```ruby
pod 'KSAdSDK', '~> 3.x.x'  # 快手广告SDK
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `KSAdSDK.framework` 或 `KSAdSDK.xcframework` |

### dcloud_properties.xml 配置

```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="ks" value="io.dcloud.feature.ad.ks.ADKsModule"/>
</feature>
```

## 9.4 Sigmob 广告

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| AdSupport.framework | 广告标识符 |
| CoreLocation.framework | 定位 |
| CoreTelephony.framework | 设备信息 |
| StoreKit.framework | 应用内购买 |
| SystemConfiguration.framework | 网络状态 |
| Security.framework | 安全服务 |
| WebKit.framework | 网页渲染 |
| libz.tbd | 压缩库 |
| libc++.tbd | C++ 运行时 |

### CocoaPods 依赖

```ruby
pod 'WindAdsSDK', '~> 4.x.x'  # Sigmob广告SDK
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `WindAds.framework` |

### dcloud_properties.xml 配置

```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="sgm" value="io.dcloud.feature.ad.sigmob.ADSMModule"/>
</feature>
```

## 9.5 百度广告

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| CoreLocation.framework | 定位 |
| CoreTelephony.framework | 设备信息 |
| SystemConfiguration.framework | 网络状态 |
| AdSupport.framework | 广告标识符 |
| SafariServices.framework | Safari服务 |
| libz.tbd | 压缩库 |

### CocoaPods 依赖

```ruby
pod 'BaiduMobAdSDK', '~> 5.x.x'  # 百度移动广告SDK
```

### dcloud_properties.xml 配置

```xml
<feature name="Ad" value="io.dcloud.feature.ad.AdFlowFeatureImpl">
    <module name="bd" value="io.dcloud.feature.ad.bd.ADBDModule" />
</feature>
```

## 9.6 其他广告平台

| 广告平台 | 所需文件 | 备注 |
|---------|---------|------|
| **华为广告** | `ads-hw-release.aar` (iOS为framework) | 需 HMS Core |
| **Pangle (穿山甲国际版)** | `PangleAdsSDK.framework` | 海外市场 |
| **Unity Ads** | `UnityAds.framework` | 游戏类应用 |
| **AppLovin** | `AppLovinSDK.framework` | 海外市场 |
| **IronSource** | `IronSourceSDK.framework` | 海外市场 |

## ⚠️ iOS 广告注意事项

1. **App Tracking Transparency (ATT)**：iOS 14.5+ 必须使用 ATT 框架请求跟踪权限，否则无法获取 IDFA
2. **SKAdNetwork**：Apple 的广告归因方案，需要在 Info.plist 中配置支持的广告网络
3. **IDFA 使用**：如需使用 IDFA 进行精准投放，必须在提交审核时选择正确的理由
4. **广告加载时机**：建议在合适的时机预加载广告，避免影响用户体验
5. **儿童隐私**：如面向儿童用户，需遵守 COPPA 法规，不得使用个性化广告

**SKAdNetwork 配置示例（Info.plist）：**
```xml
<key>SKAdNetworkItems</key>
<array>
    <dict>
        <key>SKAdNetworkIdentifier</key>
        <string>cstr6suwn9.skadnetwork</string>  <!-- 穿山甲 -->
    </dict>
    <dict>
        <key>SKAdNetworkIdentifier</key>
        <string>238da6jt44.skadnetwork</string>  <!-- 优量汇 -->
    </dict>
    <!-- 其他广告网络的 SKAdNetwork ID... -->
</array>
```

---

## 交叉引用

- 上一篇：[FacialRecognitionVerify（实人认证）](facial-recognition-verify.md)
- 下一篇：[UIWebview 配置](uiwebview.md)
- 相关模块：[Statistic（统计）](statistic.md)、[FAQ - Q5 ATT 权限弹窗](../faq.md#q5-ios-14-app-tracking-transparency-att-权限弹窗)
