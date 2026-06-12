# Push（消息推送 / uniPush）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/push.html

---

iOS 平台支持 uniPush 消息推送服务，集成个推 SDK 和 APNs（Apple Push Notification service）。

## 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| UserNotifications.framework | 用户通知框架，需设置为 Optional |
| Security.framework | 安全服务 |
| MobileCoreServices.framework | 系统服务 |
| SystemConfiguration.framework | 网络配置 |
| CoreLocation.framework | 位置信息 |
| AVFoundation.framework | 音视频能力 |
| CoreTelephony.framework | 核心电话框架（用于获取运营商信息） |

## 需要引入的系统库

| 系统库 | 说明 |
|--------|------|
| libc++.tbd | C++ 运行库 |
| libsqlite3.tbd | SQLite |
| libz.tbd | zlib |
| libresolv.tbd | DNS 解析 |

## Info.plist 配置

在 Info.plist 中添加以下权限和配置：

```xml
<!-- 权限声明 -->
<key>UIBackgroundModes</key>
<array>
    <string>remote-notification</string>
</array>

<!-- 个推配置 -->
<key>getui</key>
<dict>
    <key>appid</key>
    <string>%您的个推AppID%</string>
    <key>appkey</key>
    <string>%您的个推AppKey%</string>
    <key>appsecret</key>
    <string>%您的个推AppSecret%</string>
</dict>

<!-- 如果使用 FCM -->
<key>GOOGLE_APP_ID</key>
<string>%您的Google App ID%</string>
```

## CocoaPods 依赖

在 Podfile 中添加：

```ruby
pod 'GTSDK', '~> 2.x.x'  # 个推SDK
```

## 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `liblibPush.a` |
| SDK/libs | `libUniPush.a` |
| SDK/libs | `GTSDK.xcframework` |

> HBuilderX 5.0+ / SDK 5.07 中，`libUniPush.a` 已包含 `PGPushActualize`、`PGPushServerAct` 等实现，不要再同时链接 `libGeTuiPush.a`，否则模拟器 x86_64 架构会出现 duplicate symbols。

## feature.plist 配置

在 `PandoraApi.bundle/feature.plist` 中添加或覆盖 `Push` 节点：

```xml
<key>Push</key>
<dict>
    <key>autostart</key>
    <true/>
    <key>baseclass</key>
    <string>PGPush</string>
    <key>class</key>
    <string>PGPushActualize</string>
    <key>global</key>
    <true/>
    <key>server</key>
    <dict>
        <key>class</key>
        <string>PGPushServerAct</string>
        <key>identifier</key>
        <string>com.pushserver</string>
    </dict>
</dict>
```

## Objective-C 代码集成

在 AppDelegate 中初始化推送服务：

```objc
#import <UserNotifications/UserNotifications.h>

// 在 didFinishLaunchingWithOptions 中注册推送
- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    
    // 注册APNs
    if (@available(iOS 10.0, *)) {
        UNUserNotificationCenter *center = [UNUserNotificationCenter currentNotificationCenter];
        center.delegate = self;
        [center requestAuthorizationWithOptions:(UNAuthorizationOptionAlert | UNAuthorizationOptionSound | UNAuthorizationOptionBadge)
                              completionHandler:^(BOOL granted, NSError * _Nullable error) {
            if (granted) {
                dispatch_async(dispatch_get_main_queue(), ^{
                    [application registerForRemoteNotifications];
                });
            }
        }];
    } else {
        UIUserNotificationSettings *settings = [UIUserNotificationSettings settingsForTypes:
            (UIUserNotificationTypeBadge | UIUserNotificationTypeSound | UIUserNotificationTypeAlert)
                                                                               categories:nil];
        [application registerUserNotificationSettings:settings];
        [application registerForRemoteNotifications];
    }
    
    // 初始化个推SDK
    // [GeTuiSdk startSdkWithAppId:@"your_appid" config:nil delegate:self];
    
    return YES;
}

// 获取 DeviceToken
- (void)application:(UIApplication *)application didRegisterForRemoteNotificationsWithDeviceToken:(NSData *)deviceToken {
    // 将 deviceToken 传给个推SDK
    // [GeTuiSdk registerDeviceToken:deviceToken];
}
```

## dcloud_properties.xml 配置

```xml
<properties>
    <features>
        <feature name="Push" value="io.dcloud.feature.aps.APSFeatureImpl">
            <module name="unipush" value="io.dcloud.feature.unipush.GTPushService"/>
        </feature>
    </features>
    <services>
        <service name="push" value="io.dcloud.feature.aps.APSFeatureImpl"/>
    </services>
</properties>
```

## ⚠️ 重要提示

1. **APNs 证书配置**：需要在 Apple Developer 后台创建推送证书（开发环境/生产环境）
2. **后台模式**：确保 Xcode 项目中开启了 Remote notifications 后台模式
3. **个推账号**：在[个推官网](https://www.getui.com/)注册并创建应用，获取 AppID、AppKey、AppSecret
4. **FCM 可选**：如需海外推送，还需配置 Firebase Cloud Messaging

---

## 交叉引用

- 上一篇：[模块概览](../module-tutorial-ios.md)
- 下一篇：[Share（分享）](share.md)
- 相关模块：[Oauth（登录鉴权）](oauth.md)、[Statistic（统计）](statistic.md)
