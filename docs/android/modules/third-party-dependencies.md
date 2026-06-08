# 第三方 SDK 依赖说明（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 15. 第三方 SDK 依赖说明

### 默认集成依赖库

| SDK | 版本 | 备注 |
|---|---|---|
| androidx | V1.1.0 | androidx相关依赖库 |
| fastjson | v1.2.83 | JSON解析库 |
| android-gif-drawable | V1.2.23 | gif图显示 |
| 移动安全联盟OAID | V1.0.25 | oaid获取 |
| glide | V4.9.0 | 图片预览 |
| fresco | V1.13.0 | nvue图片展示 |
| webkit | V1.3.0 | 暗黑模式支持 |

### 其他功能模块依赖库

| SDK | 版本 | 使用模块 |
|---|---|---|
| 个推push | V3.3.7.0 | unipush |
| 百度定位 | V7.5.0 | 定位 |
| 百度地图 | V5.4.1 | map |
| 高德定位 | V6.4.5 | 定位 |
| 高德地图 | V10.0.700 | map |
| 微信 | V6.8.0 | 登录/分享/支付 |
| 新浪微博 | V12.5.0 | 登录/分享 |
| QQ | V3.5.12 | 登录/分享 |
| 友盟统计 | V9.6.1 | 统计 |
| 百度语音 | V3.4.1.101 | 语音 |
| LiteAVSDK | V6.3.7089 | livepusher |
| 腾讯x5内核 | V4.3.0.1148_43697 | X5 |
| hms | V6.13.0.301 | 华为push |
| agcp | V1.9.1.301 | 华为AGC |
| 穿山甲&GroMore | V5.7.0.5 | 广告 |
| 优量汇广告 | V4.542.1412 | 广告 |
| 快手广告联盟 | V3.3.53.3 | 广告 |
| 快手内容联盟 | V3.3.53 | 广告 |
| sigmob广告 | V4.12.7 | 广告 |
| 百度广告 | V9.322 | 广告 |
| 华为广告 | V13.4.66.300 | 广告 |
| Pangle广告 | V5.0.0.3 | 广告 |
| google AdMob | V21.4.0 | 广告 |
| ijkplayer | V0.8.8 | 视频播放 |
| DanmakuFlameMaster | V0.6.2 | 弹幕 |
| lame | V3.100 | 音频录音(MP3) |
| play-services-auth | V19.2.0 | Google登录 |
| facebook-android-sdk | V16.1.3 | Facebook登录 |

---

### 相关模块

- [Push 消息推送](push.md) — 个推push集成
- [Geolocation 定位](geolocation.md) — 百度/高德定位集成
- [Map 地图](map.md) — 百度/高德地图集成
- [Oauth 登录鉴权](oauth.md) — 微信/QQ/微博/Google/Facebook登录
- [Share 分享](share.md) — 微信/QQ/微博分享
- [Payment 支付](payment.md) — 支付宝/微信/PayPal/Stripe/Google支付
- [Speech 语音输入](speech.md) — 百度语音集成
- [Statistic 统计](statistic.md) — 友盟统计集成
- [uni-AD 广告](uni-ad.md) — 各广告平台集成
- [其他模块及国际化配置](other-modules.md) — 其他功能模块（视频/直播等）
