# Oauth（登录鉴权）Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/oauth.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/oauth.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 登录能力 | Pod subspec | 说明 |
| --- | --- | --- |
| 登录基础模块 | `Oauth` | 登录公共模块 |
| 一键登录 | `Oauth-Univerify` | 依赖 `Oauth` |
| 新浪微博登录 | `Oauth-Sina` | 依赖 `Oauth` |
| QQ 登录 | `Oauth-QQ` | 依赖 `Oauth` |
| 微信登录 | `Oauth-Wechat` | 不包含微信支付 SDK |
| 微信登录，PaySDK 版 | `Oauth-Wechat-PaySDK` | 使用带支付能力的微信 SDK |
| Apple 登录 | `Oauth-Apple` | Sign in with Apple |
| Google 登录 | `Oauth-Google` | Google Sign-In |
| Facebook 登录 | `Oauth-Facebook` | Facebook Login |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'Oauth-Wechat',
  'Oauth-Apple',
]
```

## 仍需按官方文档配置

- 一键登录需要在 DCloud 开发者中心申请 appid，并配置 `Info.plist` 中的 `DCloudConfig -> univerify -> appid`。
- 微博、QQ、微信、Google、Facebook 需要分别配置平台 AppID/AppKey、URL Schemes、白名单、Universal Links 或反向 ClientID。
- Apple 登录需要在 Xcode 能力中开启 Sign in with Apple。
- 只有同时需要微信支付能力时才使用 `Oauth-Wechat-PaySDK`；不需要支付能力时使用 `Oauth-Wechat`。
- 除 Apple 登录外，其他三方登录通常需要在 AppDelegate 的 URL / Universal Links 回调中调用 DCloud 框架方法。
