# Share（分享）Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/share.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/share.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 分享能力 | Pod subspec | 说明 |
| --- | --- | --- |
| 分享基础模块 | `Share` | 分享公共模块 |
| 新浪微博分享 | `Share-Sina` | 依赖 `Share` |
| QQ 分享 | `Share-QQ` | 依赖 `Share` |
| 微信分享 | `Share-Wechat` | 不包含微信支付 SDK |
| 微信分享，PaySDK 版 | `Share-Wechat-PaySDK` | 使用带支付能力的微信 SDK |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'Share-Wechat',
]
```

## 仍需按官方文档配置

- 分享平台账号、AppID/AppKey、URL Schemes、白名单、Universal Links 等仍需按对应平台配置。
- 只有同时需要微信支付能力时才使用 `Share-Wechat-PaySDK`；不需要支付能力时使用 `Share-Wechat`。
- 微博、QQ、微信分享均需要在 AppDelegate 的系统 URL / Universal Links 回调中调用 DCloud 框架方法。
