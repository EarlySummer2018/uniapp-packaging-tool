# iOS 注意事项（FAQ）（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

本文档汇总了 iOS 离线原生打包过程中常见的 10 个问题及解决方案。

---

## Q1: 有时安装应用之后，发现项目资源没更新

**A:** 可能是 control.xml 文件配置了 syncDebug="true" 导致的，需要改成 false 或者删除这个配置。

**解决方案：**
```xml
<!-- 将 syncDebug 改为 false 或删除该属性 -->
<control syncDebug="false">
    ...
</control>
```

---

## Q2: 更新SDK后编译报 'Could not find or use auto-linked library 'swiftXXX'' 的错误

**A:** 可能是工程为纯 OC 的项目，部分 SDK 更新后需要 swift 环境导致的，主工程添加 swift 环境即可解决。

**解决方案：**
1. 在 Xcode 项目中新建一个空的 Swift 文件（File > New > File > Swift File）
2. Xcode 会自动弹出是否创建 Bridging Header 的提示，点击 "Create Bridging Header"
3. 重新编译项目

或者手动添加：
- Build Settings > Swift Language Version 设置为 Swift 5（或更高版本）
- Build Settings > Always Embed Swift Standard Libraries 设置为 Yes

---

## Q3: 编译报错 'Building for iOS, but the linked and embedded framework 'xxx.framework' was built for iOS + iOS Simulator.'

**A:** 问题原因是依赖库中有模拟器 + 真机多架构的二进制文件。Xcode 12.3 起，Apple 不建议在一个 .framework 文件中绑定多平台的库，建议使用 .xcframework 文件替代。

**解决方案：**

**方案一：** 在 Xcode 中，进入 **TARGETS > Project Name > Build Settings > Build Options** 菜单，将 **Validate Workspace** 设置为 **Yes**。

**方案二：** 使用 lipo 命令分离架构：
```bash
# 查看当前架构
lipo -info xxx.framework/xxx

# 只保留真机架构
lipo -output xxx.framework/xxx-thin \
     -thin arm64 \
     xxx.framework/xxx

mv xxx.framework/xxx-thin xxx.framework/xxx
```

**方案三（推荐）：** 联系 SDK 提供方获取 .xcframework 格式的库文件。

---

## Q4: 升级 Xcode 15 后编译报错提示文件重复添加，或运行时闪退

**A:** 这是 Xcode 15 的链接器变更导致的兼容性问题。

**解决方案：**

在 **Build Settings > Other Linker Flags** 中添加 `-ld_classic`：

1. 打开 Xcode 项目
2. 选择 Target > Build Settings
3. 搜索 "Other Linker Flags"
4. 在 Debug 和 Release 中分别添加：`-ld_classic`
5. Clean Build Folder (Cmd + Shift + K)
6. 重新编译

**命令行方式：**
```bash
OTHER_LDFLAGS = -ld_classic
```

---

## Q5: iOS 14+ App Tracking Transparency (ATT) 权限弹窗 {#q5-ios-14-app-tracking-transparency-att-权限弹窗}

**A:** 从 iOS 14.5 开始，应用若需访问 IDFA（广告标识符），必须先请求用户授权。

**解决方案：**
```swift
import AppTrackingTransparency
import AdSupport

func requestTrackingPermission() {
    if #available(iOS 14, *) {
        ATTrackingManager.requestTrackingAuthorization { status in
            switch status {
            case .authorized:
                print("用户允许追踪")
                let idfa = ASIdentifierManager.shared().advertisingIdentifier.uuidString
                print("IDFA: \(idfa)")
            case .denied:
                print("用户拒绝追踪")
            case .notDetermined:
                print("用户未做选择")
            case .restricted:
                print("追踪受限")
            @unknown default:
                break
            }
        }
    }
}
```

**注意：** ATT 权限请求每年只能向用户展示一次，请慎重选择请求时机。

---

## Q6: iOS 17 新增配置要求

**A:** iOS 17 引入了一些新的隐私和安全要求。

**需要注意的配置项：**
1. **隐私清单 (Privacy Manifest)**：从 2024 年春季开始，所有提交到 App Store 的应用都需要包含 Privacy Manifest 文件
2. **必需的理由 API**：某些 API 需要声明使用原因
3. **后台进程限制**：进一步收紧了后台执行时间

**Privacy Manifest 示例：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSPrivacyTracking</key>
    <false/>
    <key>NSPrivacyTrackingDomains</key>
    <array/>
    <key>NSPrivacyCollectedDataTypes</key>
    <array/>
    <key>NSPrivacyAccessedAPITypes</key>
    <array>
        <dict>
            <key>NSPrivacyAccessedAPIType</key>
            <string>NSPrivacyAccessedAPICategoryFileTimestamp</string>
            <key>NSPrivacyAccessedAPITypeReasons</key>
            <array>
                <string>C617.1</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
```

---

## Q7: 真机调试证书过期或配置错误

**A:** iOS 真机调试需要有效的开发者证书和描述文件。

**排查步骤：**
1. 检查 Apple Developer 账号是否有效
2. 确认开发证书是否过期（Certificates 页面查看）
3. 确认设备的 UDID 已添加到描述文件
4. 在 Xcode 中清理缓存：Preferences > Accounts > 选择账号 > Download Manual Profiles
5. 清除 Derived Data：`rm -rf ~/Library/Developer/Xcode/DerivedData`

---

## Q8: Archive 打包失败或签名错误

**A:** 通常与证书配置或 Provisioning Profile 相关。

**常见原因及解决方案：**
1. **证书与 Bundle ID 不匹配**：确保 App ID 与 Bundle Identifier 完全一致
2. **Provisioning Profile 过期**：在 Developer Portal 重新生成
3. **Entitlements 文件缺失或错误**：检查 .entitlements 文件配置
4. **Keychain Access 问题**：解锁 Keychain 并信任对应证书

**快速修复命令：**
```bash
# 清理所有派生数据
rm -rf ~/Library/Developer/Xcode/DerivedData

# 重启模拟器服务
killall -9 Simulator

# 清理 Xcode 缓存
defaults delete com.apple.dt.Xcode
```

---

## Q9: 内存警告和应用崩溃 {#q9-内存警告和应用崩溃}

**A:** iOS 对内存管理非常严格，特别是在使用 WebView、图片处理、音视频等功能时。

**优化建议：**
1. **WebView 内存泄漏**：及时释放 WebView，避免循环引用
2. **大图处理**：使用 ImageIO 或 downsample 方式加载大图
3. **缓存管理**：合理设置内存缓存大小，收到内存警告时主动清理
4. **僵尸对象检测**：开启 Zombies 检测内存访问问题

```objc
// 监听内存警告
[[NSNotificationCenter defaultCenter] addObserver:self
                                         selector:@selector(handleMemoryWarning:)
                                             name:UIApplicationDidReceiveMemoryWarningNotification
                                           object:nil];

- (void)handleMemoryWarning:(NSNotification *)notification {
    // 清理缓存、释放不必要的资源
    [[NSURLCache sharedURLCache] removeAllCachedResponses];
    // 清理图片缓存
    // ...
}
```

---

## Q10: 网络请求失败或 SSL 错误

**A:** iOS 9 起，默认禁止 HTTP 明文传输，强制使用 HTTPS。

**解决方案：**

**方案一（临时，仅开发环境）：**
```xml
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>
```

**方案二（生产环境推荐）：配置例外域名**
```xml
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSExceptionDomains</key>
    <dict>
        <key>your-api-domain.com</key>
        <dict>
            <key>NSIncludesSubdomains</key>
            <true/>
            <key>NSExceptionAllowsInsecureHTTPLoads</key>
            <true/>
            <key>NSExceptionMinimumTLSVersion</key>
            <string>TLSv1.2</string>
        </dict>
    </dict>
</dict>
```

**方案三（最佳实践）：** 全站升级 HTTPS，使用有效 SSL 证书

---

## 交叉引用

- 上一篇：[Payment（支付）](modules/payment.md)
- 下一篇：[第三方 SDK 依赖说明](modules/third-party-dependencies.md)
- 相关模块：所有模块均可能遇到以上问题，请根据具体症状查阅对应条目
