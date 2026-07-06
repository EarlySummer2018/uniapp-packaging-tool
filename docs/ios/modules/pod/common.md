# HBuilderX 5.13+ 本地 Pod 集成总览

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/common.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/common.md
> 适用版本：HBuilderX 5.13+ iOS 离线 SDK

## 官方集成方式

HBuilderX 5.13+ 的 iOS 离线 SDK 根目录内置 `uniapp.podspec`。官方推荐在示例工程 `HBuilder-Hello` 的 `Podfile` 中维护 `uniapp_subspecs`，通过本地 CocoaPods 集成基础能力和三方模块。

```ruby
platform :ios, '13.0'
project 'HBuilder-Hello.xcodeproj'

require_relative 'scripts/uniapp_module_config'
require_relative 'uniapp_config' if File.exist?(File.join(__dir__, 'uniapp_config.rb'))

uniapp_subspecs = [
  'Core',
  'Barcode',
  'CameraGallery',
  'Payment-Wechat',
  'Map-Gaode',
  'UniAd-GDT',
]

target 'HBuilder' do
  pod 'uniapp', :path => '..', :subspecs => uniapp_subspecs
end

post_install do |_installer|
  UniAppModuleConfig.apply(
    uniapp_subspecs,
    plist_values: defined?(UNIAPP_PLIST_VALUES) ? UNIAPP_PLIST_VALUES : {}
  )
end
```

修改 `Podfile` 后执行：

```sh
pod install --no-repo-update
```

执行完成后使用 `.xcworkspace` 打开工程。

## 业务参数配置

官方示例工程支持通过 `uniapp_config.rb` 集中配置部分 appid、appkey、URL Scheme、Universal Links 等参数；`pod install` 时脚本会按已启用的 `uniapp_subspecs` 写入对应的 `Info.plist` 和部分 `feature.plist` 配置。

```ruby
UNIAPP_PLIST_VALUES = {
  payment_wechat: {
    appid: '微信 AppID',
    universal_links: 'https://example.com/app/'
  },
  payment_alipay: {
    scheme: '支付宝回调 Scheme'
  },
  map_gaode: {
    appkey: '高德地图 AppKey'
  },
  statistic_umeng: {
    appkey: '友盟 AppKey',
    channel: 'App Store'
  },
  uniad: {
    market_channel: 'io.dcloud.HBuilder|appid|adid|apple',
    dcloud_ad_id: '广告标识 adid'
  }
}.freeze
```

官方说明里仍需要手工处理的内容包括：证书/Profile、三方平台后台配置、`GoogleService-Info.plist`、AppDelegate 回调、`UniAd-WM` 的微信参数，以及移除模块后历史写入的 `Info.plist`、`feature.plist`、entitlements 配置。

## Pod subspec 对照

`Core` 是基础运行模块，必须保留。带平台后缀的 subspec 通常会通过 podspec 依赖带上对应基础模块。

| 功能模块 | Pod subspec | 说明 |
| --- | --- | --- |
| 基础运行模块 | `Core` | 必须保留 |
| 加速度计 | `Accelerometer` | 加速度传感器 |
| 音频 | `Audio` | 录音、音频相关能力 |
| 相机/相册 | `CameraGallery` | 拍照、选择图片/视频 |
| 通讯录 | `Contacts` | 通讯录能力 |
| 文件系统 | `File` | 文件读写 |
| 短彩邮件消息 | `Messaging` | 系统短信、邮件等消息能力 |
| 屏幕方向 | `Orientation` | 设备方向 |
| 距离传感器 | `Proximity` | 距离传感器 |
| 网络请求 | `XMLHttpRequest` | XHR 网络请求 |
| 压缩解压 | `Zip` | zip 能力 |
| 扫码 | `Barcode` | 条码/二维码扫描 |
| Canvas | `Canvas` | Canvas 能力 |
| 视频播放 | `Video` | video 播放能力 |
| 指纹识别 | `Fingerprint` | Touch ID |
| Face ID | `FaceId` | Face ID |
| 蓝牙 | `BlueTooth` | 蓝牙能力 |
| SQLite | `Sqlite` | SQLite 数据库 |
| iBeacon | `IBeacon` | iBeacon |
| 日志 | `Log` | 输出 `console.log()` 等日志 |
| 系统定位 | `Geolocation` | 系统定位基础模块 |
| 百度定位 | `Geolocation-Baidu` | 依赖 `Geolocation` |
| 高德定位 | `Geolocation-Gaode` | 依赖 `Geolocation` |
| 百度地图 | `Map-Baidu` | 百度地图 |
| 高德地图 | `Map-Gaode` | 高德地图 |
| Google 地图 | `Map-Google` | Google Maps |
| 登录基础模块 | `Oauth` | 登录公共模块 |
| 一键登录 | `Oauth-Univerify` | 依赖 `Oauth` |
| 新浪微博登录 | `Oauth-Sina` | 依赖 `Oauth` |
| QQ 登录 | `Oauth-QQ` | 依赖 `Oauth` |
| 微信登录 | `Oauth-Wechat` | 不包含微信支付 SDK |
| 微信登录，PaySDK 版 | `Oauth-Wechat-PaySDK` | 使用带支付能力的微信 SDK |
| Apple 登录 | `Oauth-Apple` | Sign in with Apple |
| Google 登录 | `Oauth-Google` | Google Sign-In |
| Facebook 登录 | `Oauth-Facebook` | Facebook Login |
| 支付基础模块 | `Payment` | 支付公共模块 |
| 支付宝支付 | `Payment-AliPay` | 依赖 `Payment` |
| 微信支付 | `Payment-Wechat` | 依赖 `Payment` |
| Apple IAP | `Payment-IAP` | 依赖 `Payment` |
| PayPal 支付 | `Payment-Paypal` | 依赖 `Payment` |
| Stripe 支付 | `Payment-Stripe` | 依赖 `Payment` |
| 推送基础模块 | `Push` | 推送公共模块 |
| UniPush / 个推 | `Push-UniPush` | 依赖 `Push` |
| 个推 | `Push-Getui` | 依赖 `Push` |
| FCM 推送 | `Push-FCM` | 通常还需添加 `GoogleService-Info.plist` |
| 分享基础模块 | `Share` | 分享公共模块 |
| 新浪微博分享 | `Share-Sina` | 依赖 `Share` |
| QQ 分享 | `Share-QQ` | 依赖 `Share` |
| 微信分享 | `Share-Wechat` | 不包含微信支付 SDK |
| 微信分享，PaySDK 版 | `Share-Wechat-PaySDK` | 使用带支付能力的微信 SDK |
| 语音基础模块 | `Speech` | 语音公共模块 |
| 百度语音 | `Speech-Baidu` | 依赖 `Speech` |
| 讯飞语音 | `Speech-Ifly` | 依赖 `Speech` |
| 直播推流 | `LivePusher` | 直播推流 |
| 统计基础模块 | `Statistic` | 统计公共模块 |
| 友盟统计 | `Statistic-Umeng` | 依赖 `Statistic` |
| Firebase 统计 | `Statistic-Firebase` | 通常还需添加 `GoogleService-Info.plist` |
| UIWebView 兼容 | `UIWebview` | 兼容模块 |
| 实人认证 | `FacialRecognitionVerify` | 人脸/实人认证 |
| UTS 基础模块 | `UTS` | UTS 运行支持 |
| uni-AD 穿山甲 | `UniAd-CSJ` | 自动依赖 `UniAd-Base` |
| uni-AD Gromore | `UniAd-Gromore` | 自动依赖 `UniAd-Base` |
| uni-AD 优量汇 | `UniAd-GDT` | 自动依赖 `UniAd-Base` |
| uni-AD 快手 | `UniAd-KS` | 自动依赖 `UniAd-Base` |
| uni-AD Sigmob | `UniAd-Sigmob` | 自动依赖 `UniAd-Base` |
| uni-AD 百度 | `UniAd-Baidu` | 自动依赖 `UniAd-Base` |
| uni-AD 微信小程序广告 | `UniAd-WM` | 自动依赖 `UniAd-Base` |
| uni-AD 旺脉 | `UniAd-WA` | 自动依赖 `UniAd-Base` |
| uni-AD AppLovin | `UniAd-AppLovin` | 自动依赖 `UniAd-Base` |
| uni-AD AdMob | `UniAd-GG` | 自动依赖 `UniAd-Base` |
| uni-AD AdMob Pangle Adapter | `UniAd-GG-Pangle` | 自动依赖 `UniAd-Base` |
| uni-AD Gromore 短剧 | `UniAd-GM-Content` | 示例工程已配置所需 CocoaPods source |
| uni-AD InMobi | `UniAd-InMobi` | 自动依赖 `UniAd-Base` |
| uni-AD IronSource | `UniAd-IronSource` | 自动依赖 `UniAd-Base` |
| uni-AD 快手内容联盟 | `UniAd-KS-Content` | 自动依赖 `UniAd-Base` |
| uni-AD Liftoff / Vungle | `UniAd-Liftoff` | 自动依赖 `UniAd-Base` |
| uni-AD Meta | `UniAd-Meta` | 自动依赖 `UniAd-Base` |
| uni-AD Mintegral | `UniAd-Mintegral` | 自动依赖 `UniAd-Base` |
| uni-AD Pangle | `UniAd-Pangle` | 自动依赖 `UniAd-Base` |
| uni-AD UnityAds | `UniAd-Unity` | 自动依赖 `UniAd-Base` |
| uni-AD Oct | `UniAd-Oct` | 自动依赖 `UniAd-Base` |
| uni-AD 泛连 | `UniAd-FL` | 自动依赖 `UniAd-Base` |
| uni-AD 华夏乐游 | `UniAd-YT` | 自动依赖 `UniAd-Base` |
