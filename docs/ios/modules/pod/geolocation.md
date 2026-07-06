# Geolocation（定位）Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/geolocation.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/geolocation.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 定位能力 | Pod subspec | 说明 |
| --- | --- | --- |
| 系统定位 | `Geolocation` | 系统定位基础模块 |
| 百度定位 | `Geolocation-Baidu` | 依赖 `Geolocation` |
| 高德定位 | `Geolocation-Gaode` | 依赖 `Geolocation` |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'Geolocation-Gaode',
]
```

## 仍需按官方文档配置

- 百度定位需要在百度地图开放平台申请 AppKey，并配置 `Info.plist` 中的 `baidu` 节点。
- 高德定位需要在高德开放平台申请 AppKey，并配置 `Info.plist` 中的 `amap` 节点。
- 系统定位、百度定位、高德定位都需要按官方页面补齐定位隐私权限描述。
- 如只使用地图模块中的定位能力，先确认是否需要 `Map-*`，避免重复启用不必要模块。
