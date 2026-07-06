# Payment（支付）Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/pay.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/pay.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 支付能力 | Pod subspec | 说明 |
| --- | --- | --- |
| 支付基础模块 | `Payment` | 支付公共模块 |
| 支付宝支付 | `Payment-AliPay` | 依赖 `Payment` |
| 微信支付 | `Payment-Wechat` | 依赖 `Payment` |
| Apple IAP | `Payment-IAP` | 依赖 `Payment` |
| PayPal 支付 | `Payment-Paypal` | 依赖 `Payment` |
| Stripe 支付 | `Payment-Stripe` | 依赖 `Payment` |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'Payment-AliPay',
  'Payment-Wechat',
]
```

## 仍需按官方文档配置

- 支付平台账号、商户配置、URL Schemes、白名单、Universal Links、returnUrl 等仍需按对应支付平台配置。
- 支付宝可通过 `uniapp_config.rb` 中的 `payment_alipay.scheme` 写入回调 Scheme。
- 微信支付可通过 `uniapp_config.rb` 中的 `payment_wechat.appid` 和 `payment_wechat.universal_links` 写入部分配置。
- 除 Apple IAP 外，其他支付通常需要在 AppDelegate 的 URL 回调中调用 DCloud 框架方法。
- 微信 SDK 分普通版和 PaySDK 版；不使用支付能力的分享/登录场景不要引入 PaySDK 版。
