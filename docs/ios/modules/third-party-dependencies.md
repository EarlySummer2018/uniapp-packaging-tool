# 第三方 SDK 依赖说明（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

以下是 iOS 平台常用的第三方 SDK 及其版本信息汇总表。

## 默认集成依赖库

| SDK 名称 | 版本 | HBuilderX 最低版本 | 说明 |
|---------|------|-------------------|------|
| **AFNetworking** | ~4.0 | V3.7.0 | 网络请求库 |
| **SDWebImage** | ~5.12 | V3.7.0 | 图片异步加载和缓存 |
| **Masonry** | ~1.1 | V3.7.0 | Auto Layout 封装 |
| **MJRefresh** | ~3.7 | V3.7.0 | 下拉刷新控件 |
| **YYModel** | ~1.0 | V3.7.0 | JSON 转 Model |
| **FMDB** | ~2.7 | V3.7.0 | SQLite 封装 |
| **SSZipArchive** | ~2.4 | V3.7.0 | 压缩解压工具 |

## 功能模块依赖库

| SDK 名称 | 版本 | HBuilderX 最低版本 | 使用模块 | 说明 |
|---------|------|-------------------|---------|------|
| **个推 GTSDK** | ~2.x | V3.3.1 | unipush | 消息推送 |
| **微信 WechatOpenSDK** | 1.9.2 | V3.7.12 | Oauth/Share/Payment | 微信生态 |
| **QQ TencentOpenAPI** | 3.5.x | V3.5.5 | Oauth/QQ | QQ 开放平台 |
| **新浪微博 WeiboSDK** | ~2.5 | V3.6.1 | Oauth/Share | 微博开放平台 |
| **百度地图 BaiduMapKit** | ~7.5 | V2.0.0 | Map | 百度地图 |
| **高德地图 AMap3DMap** | ~10.0 | V4.18 | Map | 高德地图 |
| **百度语音 BDSpeechSDK** | ~3.x | V3.0.1 | Speech | 语音识别 |
| **讯飞语音 iflyMSC** | ~1.x | V3.0.1 | Speech | 讯飞语音 |
| **腾讯直播 TXLiteAVSDK** | ~11.x | V3.0.1 | LivePusher | 直播推流 |
| **友盟 UMCommon** | ~7.x | V3.8.3 | Statistic | 友盟统计 |
| **Firebase Analytics** | ~10.x | V3.2.7 | Statistic | 谷歌分析 |
| **穿山甲 Bytedance-UnionSDK** | ~5.x | V3.98 | uni-AD | 字节广告 |
| **优量汇 GDTMobSDK** | ~4.x | V3.93 | uni-AD | 腾讯广告 |
| **快手 KSAdSDK** | ~3.x | V3.93 | uni-AD | 快手广告 |
| **Sigmob WindAdsSDK** | ~4.x | V3.93 | uni-AD | Sigmob 广告 |
| **百度移动广告 BaiduMobAdSDK** | ~5.x | V3.93 | uni-AD | 百度广告 |
| **支付宝 AlipaySDK-iOS** | ~15.8 | V3.0.1 | Payment | 支付宝支付 |
| **Stripe iOS SDK** | ~24.x | V3.2.7 | Payment | Stripe 支付 |
| **Google Sign-In** | ~7.x | V3.2.7 | Oauth | Google 登录 |
| **Facebook SDK** | ~16.x | V3.91 | Oauth/Share | Facebook |
| **Apple AuthenticationServices** | System | V13.0+ | Oauth | Apple 登录（系统自带）|

## CocoaPods Podfile 示例

```ruby
# Uncomment the next line to define a global platform for your project
platform :ios, '12.0'

target 'YourProject' do
  # Comment the next line if you don't want to use dynamic frameworks
  use_frameworks!
  
  # Pods for YourProject
  
  # 基础库
  pod 'AFNetworking', '~> 4.0'
  pod 'SDWebImage', '~> 5.12'
  pod 'Masonry', '~> 1.1'
  
  # 推送（按需集成）
  pod 'GTSDK', '~> 2.x'           # 个推
  # pod 'FirebaseMessaging', '~> 10.x'  # FCM（可选）
  
  # 第三方登录/分享（按需集成）
  pod 'WechatOpenSDK', '1.9.2'    # 微信
  # pod 'TencentOpenApiSdk'         # QQ
  # pod 'Weibo_SDK'                 # 微博
  # pod 'GoogleSignIn', '~> 7.x'    # Google
  # pod 'FBSDKLoginKit'             # Facebook
  
  # 地图（按需集成，二选一）
  # pod 'BaiduMapKit', '~> 7.x'     # 百度地图
  # pod 'AMap3DMap', '~> 10.x'      # 高德地图
  
  # 语音识别（按需集成，二选一）
  # pod 'BDSpeechSDK', '~> 3.x'     # 百度语音
  # pod 'iflyMSC', '~> 1.x'         # 讯飞语音
  
  # 直播推流
  # pod 'TXLiteAVSDK_Professional', '~> 11.x'
  
  # 统计（按需集成，二选一）
  # pod 'UMCommon', '~> 7.x'        # 友盟
  # pod 'Firebase/Core'             # 谷歌分析
  
  # 广告（按需集成）
  # pod 'Bytedance-UnionSDK', '~> 5.x'   # 穿山甲
  # pod 'GDTMobSDK', '~> 4.x'            # 优量汇
  # pod 'KSAdSDK', '~> 3.x'              # 快手
  # pod 'WindAdsSDK', '~> 4.x'           # Sigmob
  
  # 支付（按需集成）
  # pod 'AlipaySDK-iOS', '~> 15.8'       # 支付宝
  # pod 'Stripe', '~> 24.x'              # Stripe
  
end

post_install do |installer|
  installer.pods_project.targets.each do |target|
    target.build_configurations.each do |config|
      config.build_settings['IPHONEOS_DEPLOYMENT_TARGET'] = '12.0'
    end
  end
end
```

## ⚠️ SDK 版本管理建议

1. **固定版本号**：生产环境建议锁定具体版本号，避免自动升级导致兼容性问题
2. **定期更新**：关注 SDK 的安全补丁和 bug 修复，定期评估是否需要升级
3. **版本冲突**：不同 SDK 可能依赖同一库的不同版本，使用 CocoaPods 的 resolver 解决冲突
4. **架构支持**：确保 SDK 支持 arm64（真机）和 x86_64（模拟器），或使用 xcframework
5. **废弃通知**：关注 SDK 官方的 deprecation 通知，提前规划迁移方案

---

## 交叉引用

- 上一篇：[FAQ - iOS 注意事项](faq.md)
- 回到目录：[iOS 模块配置教程总览](../module-tutorial-ios.md)
- 相关模块：所有模块的 SDK 依赖信息均汇总于此
