# uni-AD Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/uniad.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/uniad.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

启用任意广告平台 Pod 时，官方说明会自动带上广告公共依赖 `UniAd-Base`。

| 广告平台 | Pod subspec | 说明 |
| --- | --- | --- |
| 穿山甲 | `UniAd-CSJ` | 自动依赖 `UniAd-Base` |
| Gromore | `UniAd-Gromore` | 自动依赖 `UniAd-Base` |
| 腾讯优量汇 | `UniAd-GDT` | 自动依赖 `UniAd-Base` |
| 快手 | `UniAd-KS` | 自动依赖 `UniAd-Base` |
| Sigmob | `UniAd-Sigmob` | 自动依赖 `UniAd-Base` |
| 百度 | `UniAd-Baidu` | 自动依赖 `UniAd-Base` |
| 微信小程序广告 | `UniAd-WM` | 自动依赖 `UniAd-Base` |
| 旺脉 | `UniAd-WA` | 自动依赖 `UniAd-Base` |
| AppLovin | `UniAd-AppLovin` | 自动依赖 `UniAd-Base` |
| Google AdMob | `UniAd-GG` | 自动依赖 `UniAd-Base` |
| AdMob Pangle Adapter | `UniAd-GG-Pangle` | 自动依赖 `UniAd-Base` |
| Gromore 短剧 | `UniAd-GM-Content` | 示例工程已配置所需 CocoaPods source |
| InMobi | `UniAd-InMobi` | 自动依赖 `UniAd-Base` |
| IronSource | `UniAd-IronSource` | 自动依赖 `UniAd-Base` |
| 快手内容联盟 | `UniAd-KS-Content` | 自动依赖 `UniAd-Base` |
| Liftoff / Vungle | `UniAd-Liftoff` | 自动依赖 `UniAd-Base` |
| Meta Audience Network | `UniAd-Meta` | 自动依赖 `UniAd-Base` |
| Mintegral | `UniAd-Mintegral` | 自动依赖 `UniAd-Base` |
| Pangle | `UniAd-Pangle` | 自动依赖 `UniAd-Base` |
| UnityAds | `UniAd-Unity` | 自动依赖 `UniAd-Base` |
| Oct | `UniAd-Oct` | 自动依赖 `UniAd-Base` |
| 泛连 | `UniAd-FL` | 自动依赖 `UniAd-Base` |
| 华夏乐游 | `UniAd-YT` | 自动依赖 `UniAd-Base` |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'UniAd-GDT',
  'UniAd-KS',
]
```

## 仍需按官方文档配置

- 5.13+ 示例工程 `Podfile` 已配置广告 SDK 需要的 CocoaPods source；启用 uni-AD 模块后，广告三方 SDK 通过 CocoaPods 依赖集成。
- 需要在 DCloud 广告联盟申请账号并开通广告位。
- 可参考 `uniapp_config.rb` 中的 `uniad.market_channel` 和 `uniad.dcloud_ad_id` 写入广告标识。
- `UniAd-WM` 的微信参数官方要求按 uni-AD 文档手动配置。
- ATT、SKAdNetwork、隐私清单、渠道字段等合规配置仍需按官方页面逐项检查。
