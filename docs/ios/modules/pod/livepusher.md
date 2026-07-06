# LivePusher（直播推流）Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/livepusher.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/livepusher.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 能力 | Pod subspec | 说明 |
| --- | --- | --- |
| 直播推流 | `LivePusher` | 直播推流模块 |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'LivePusher',
]
```

## 仍需按官方文档配置

- 直播推流涉及相机、麦克风等权限，需按官方文档检查 `Info.plist` 隐私描述。
- 又拍云直播推流依赖的底层 SDK 以官方页面说明为准；旧版手动依赖表仍可用于排查动态库、真机调试和架构问题。
- 若项目同时接入其他音视频 SDK，需按官方注意事项排查符号冲突。
