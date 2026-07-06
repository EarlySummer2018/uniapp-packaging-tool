# Statistic（统计）Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/statistic.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/statistic.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 统计能力 | Pod subspec | 说明 |
| --- | --- | --- |
| 统计基础模块 | `Statistic` | 统计公共模块 |
| 友盟统计 | `Statistic-Umeng` | 依赖 `Statistic` |
| Firebase 统计 | `Statistic-Firebase` | 通常还需添加 `GoogleService-Info.plist` |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'Statistic-Umeng',
]
```

## 仍需按官方文档配置

- 友盟统计需要申请 AppKey；可参考 `uniapp_config.rb` 中的 `statistic_umeng.appkey` 和 `statistic_umeng.channel`。
- `Statistic-Firebase` 通常需要添加 Firebase 生成的 `GoogleService-Info.plist`。
- IDFA、隐私清单、ATT 等合规配置仍需按官方统计模块和 Apple 要求处理。
