# iOS UIWebView Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/uiwebview.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/uiwebview.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 能力 | Pod subspec | 说明 |
| --- | --- | --- |
| UIWebView 兼容 | `UIWebview` | UIWebView 兼容模块 |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'UIWebview',
]
```

## 仍需按官方文档配置

- 官方文档说明 UIWebView 已从基础引擎中移除，变成可选模块。
- 只有确实需要 UIWebView 兼容能力时才启用 `UIWebview`。
- App Store 对 UIWebView API 有审核风险，启用前需确认业务必要性。
