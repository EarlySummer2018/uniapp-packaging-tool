# 第三方 SDK 依赖与本地 Pod

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/dependentLibrary.html
> 相关总表：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/common.html
> 适用版本：HBuilderX 5.13+

## 5.13+ 处理原则

官方 5.13+ 本地 Pod 方式会通过 `uniapp.podspec` 自动处理大部分 `.a`、`.framework/.xcframework`、系统库和资源文件依赖。第三方 SDK 依赖说明页主要用于旧版手动集成或依赖排查。

## 不建议重复添加

已通过 `uniapp_subspecs` 启用的内置模块，不建议再按旧版依赖表重复手工添加对应三方 SDK，避免重复符号、架构冲突或版本不一致。

## 仍需手工确认的常见项

- `GoogleService-Info.plist`：`Push-FCM`、`Statistic-Firebase` 等 Firebase 相关模块通常仍需添加。
- CocoaPods source：官方 uni-AD 页面说明 5.13+ 示例工程 `Podfile` 已配置广告 SDK 需要的 source。
- 业务后台参数：支付、登录、分享、地图、推送、统计、广告等模块仍需按各平台后台配置。
- 合规文件：Privacy Manifest、ATT、SKAdNetwork、entitlements、证书/Profile 等仍需按官方模块文档和 Apple 要求检查。

## 排查路径

1. 先确认 `Podfile` 的 `uniapp_subspecs` 是否启用了正确模块。
2. 执行 `pod install --no-repo-update` 并使用 `.xcworkspace` 打开工程。
3. 若仍有链接或资源问题，再对照官方第三方 SDK 依赖页和对应模块页检查缺失项。
