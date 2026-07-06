# Speech（语音输入）Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/speech.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/speech.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 语音能力 | Pod subspec | 说明 |
| --- | --- | --- |
| 语音基础模块 | `Speech` | 语音公共模块 |
| 百度语音 | `Speech-Baidu` | 依赖 `Speech` |
| 讯飞语音 | `Speech-Ifly` | 依赖 `Speech` |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'Speech-Baidu',
]
```

## 仍需按官方文档配置

- 百度语音需要到百度语音平台创建应用并获取 appkey 等信息。
- 讯飞语音需要按讯飞平台要求配置对应账号参数。
- 麦克风、语音识别等隐私权限描述仍需在 `Info.plist` 中按官方文档补齐。
