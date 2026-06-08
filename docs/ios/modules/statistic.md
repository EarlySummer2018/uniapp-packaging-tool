# Statistic（统计）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

iOS 统计模块支持友盟统计和 Google Analytics。

## 7.1 友盟统计

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| CoreTelephony.framework | 设备信息 |
| Security.framework | 数据安全 |
| SystemConfiguration.framework | 网络状态 |
| libz.tbd | 数据压缩 |
| libsqlite3.tbd | 本地存储 |
| libc++.tbd | C++ 运行时 |

### Info.plist 配置

```xml
<!-- 友盟 AppKey -->
<key>UMENG_APPKEY</key>
<string>%友盟AppKey%</string>

<!-- 渠道号（iOS 通常为 App Store） -->
<key>UMENG_CHANNEL</key>
<string>App Store</string>
```

### CocoaPods 依赖

```ruby
pod 'UMCommon', '~> 7.x.x'      # 友盟核心库
pod 'UMAnalytics', '~> 9.x.x'   # 友盟统计分析
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `UMCommon.framework`, `UMAnalytics.framework` 等 |

### Objective-C 代码初始化

```objc
#import <UMCommon/UMCommon.h>
#import <UMAnalytics/MobClick.h>

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    
    // 初始化友盟统计
    [UMConfigure setLogEnabled:NO];  // 关闭日志（上线时应关闭）
    [UMConfigure initWithAppkey:@"您的友盟AppKey" channel:@"App Store"];
    
    // 自动页面采集（可选）
    [MobClick setAutoPageEnabled:YES];
    
    return YES;
}
```

### dcloud_properties.xml 配置

```xml
<features>
    <feature name="Statistic" value="io.dcloud.feature.statistics.StatisticsFeatureImpl">
        <module name="Statistic-Umeng" value="io.dcloud.feature.statistics.umeng.UmengStatistics" />
    </feature>
</features>
<services>
    <service name="Statistic-Umeng" value="io.dcloud.feature.statistics.umeng.StatisticsBootImpl"/>
</services>
```

## 7.2 Google Analytics（Firebase）

### 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| FirebaseAnalytics.framework | Firebase 分析框架 |
| FirebaseInstanceID.framework | 实例ID框架 |
| GoogleUtilities.framework | Google 工具库 |
| nanopb.framework | Protocol Buffers 库 |

### CocoaPods 依赖

```ruby
pod 'Firebase/Core'
pod 'Firebase/Analytics'
```

### 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| 项目根目录 | `GoogleService-Info.plist`（从 Firebase 控制台下载） |

### Objective-C 代码初始化

```objc
#import <Firebase/Firebase.h>

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    
    // 配置 Firebase
    [FIRApp configure];
    
    return YES;
}
```

### dcloud_properties.xml 配置

```xml
<feature name="Statistic" value="io.dcloud.feature.statistics.StatisticsFeatureImpl">
    <module name="Statistic-Google" value="io.dcloud.feature.statistics.google.GoogleStatistics" />
</feature>
```

## ⚠️ 统计注意事项

1. **隐私合规**：收集用户数据前必须获得用户同意（GDPR/CCPA 等）
2. **数据上报策略**：建议设置合理的上报间隔，避免频繁请求
3. **渠道追踪**：不同分发渠道应使用不同的 channel 参数
4. **调试模式**：开发阶段可开启日志，上线前务必关闭

---

## 交叉引用

- 上一篇：[LivePusher（直播推流）](livepusher.md)
- 下一篇：[FacialRecognitionVerify（实人认证）](facial-recognition-verify.md)
- 相关模块：[uni-AD（广告）](uni-ad.md)（同样涉及隐私合规）、[Push（消息推送）](push.md)（可结合推送+统计）
