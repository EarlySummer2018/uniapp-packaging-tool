# iOS 5.13+ Pod 集成文档索引

> 官方总入口：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/common.html
> 适用版本：HBuilderX 5.13+ iOS 离线 SDK
> 本目录只整理官方 5.13+ 本地 Pod 集成方式；旧版手动添加 `.a`、`.framework/.xcframework`、系统库和资源文件的依赖表仍以各模块官方页面为准。

## 使用顺序

1. 先阅读 [common.md](common.md)，确认 `uniapp.podspec`、`uniapp_subspecs`、`uniapp_config.rb`、`pod install --no-repo-update` 的通用用法。
2. 再按实际启用能力阅读对应模块文档。
3. 修改 `Podfile` 后执行 `pod install --no-repo-update`，并使用 `.xcworkspace` 打开工程。
4. appid、appkey、URL Scheme、Universal Links、证书/Profile、AppDelegate 回调、`GoogleService-Info.plist` 等业务配置仍需按对应模块官方文档处理。

## 模块文档

| 文档 | 内容 |
| --- | --- |
| [common.md](common.md) | 5.13+ 本地 Pod 通用集成方式和完整 subspec 对照 |
| [base-modules.md](base-modules.md) | 基础运行模块和非三方基础能力 |
| [geolocation.md](geolocation.md) | 系统定位、百度定位、高德定位 |
| [map.md](map.md) | 百度地图、高德地图、Google 地图 |
| [oauth.md](oauth.md) | 一键登录、微博、QQ、微信、Apple、Google、Facebook 登录 |
| [payment.md](payment.md) | 支付宝、微信、Apple IAP、PayPal、Stripe 支付 |
| [push.md](push.md) | Push、UniPush/个推、FCM |
| [share.md](share.md) | 微博、QQ、微信分享 |
| [speech.md](speech.md) | 百度语音、讯飞语音 |
| [livepusher.md](livepusher.md) | 直播推流 |
| [statistic.md](statistic.md) | 友盟统计、Firebase 统计 |
| [facial-recognition-verify.md](facial-recognition-verify.md) | 实人认证 |
| [uni-ad.md](uni-ad.md) | uni-AD 各广告平台 |
| [uiwebview.md](uiwebview.md) | UIWebView 兼容模块 |
| [uts-builtin-modules.md](uts-builtin-modules.md) | UTS 基础模块、UTS 插件自动集成 |
| [uts-plugin-build.md](uts-plugin-build.md) | UTS 插件 5.13+ Pod 自动集成说明 |
| [native-plugins.md](native-plugins.md) | 原生插件与 CocoaPods 的边界说明 |
| [third-party-dependencies.md](third-party-dependencies.md) | 第三方 SDK 依赖与本地 Pod 的关系 |
