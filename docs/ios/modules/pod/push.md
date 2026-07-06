# Push（消息推送 / uniPush）Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/push.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/push.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 推送能力 | Pod subspec | 说明 |
| --- | --- | --- |
| 推送基础模块 | `Push` | 推送公共模块 |
| UniPush / 个推 | `Push-UniPush` | 依赖 `Push` |
| 个推 | `Push-Getui` | 依赖 `Push` |
| FCM 推送 | `Push-FCM` | 通常还需添加 `GoogleService-Info.plist` |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'Push-UniPush',
]
```

## 仍需按官方文档配置

- APNs 证书、Profile、推送权限、后台 Remote notifications 能力仍需在 Apple Developer 和 Xcode 中配置。
- 个推需要在个推后台创建应用并填写 AppID、AppKey、AppSecret。
- `Push-FCM` 通常需要添加 Firebase 生成的 `GoogleService-Info.plist`。
- AppDelegate 中 APNs 注册、deviceToken 传递等回调逻辑仍按官方推送文档处理。
