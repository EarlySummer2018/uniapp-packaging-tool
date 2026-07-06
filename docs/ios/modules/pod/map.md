# Map（地图）Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/map.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/map.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 地图能力 | Pod subspec | 说明 |
| --- | --- | --- |
| 百度地图 | `Map-Baidu` | 百度地图 |
| 高德地图 | `Map-Gaode` | 高德地图 |
| Google 地图 | `Map-Google` | Google Maps |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'Map-Gaode',
]
```

## 仍需按官方文档配置

- 只使用定位能力时，可改用 `Geolocation`、`Geolocation-Baidu` 或 `Geolocation-Gaode`。
- 百度、高德、Google 地图都需要到对应开放平台申请 AppKey/APIKey，并配置 `Info.plist`。
- 官方地图页提示工程里只能保留一个地图方案；切换地图方案时需要清理旧方案相关的 `Info.plist` key 和库文件配置。
- 使用高德地图等业务参数时，可参考官方 5.13+ 示例工程中的 `uniapp_config.rb` 集中写入。
