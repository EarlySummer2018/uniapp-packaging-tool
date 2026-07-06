# FacialRecognitionVerify（实人认证）Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/facialRecognitionVerify.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/facialRecognitionVerify.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 能力 | Pod subspec | 说明 |
| --- | --- | --- |
| UTS 基础模块 | `UTS` | 实人认证依赖 UTS 基础能力 |
| 实人认证 | `FacialRecognitionVerify` | 人脸/实人认证 |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'UTS',
  'FacialRecognitionVerify',
]
```

## 仍需按官方文档配置

- 官方说明中实人认证依赖 UTS 基础能力；集成前需确认 `UTS` 能力可用。
- 需按实人认证业务文档开通服务并配置对应云端/业务参数。
- 相机等隐私权限仍需按官方文档补齐。
- 旧版手动依赖表可用于排查阿里云实人认证相关 framework 和 bundle 是否完整。
