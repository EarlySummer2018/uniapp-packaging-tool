# 原生插件与 CocoaPods 集成边界

> 官方文档：https://ask.dcloud.net.cn/article/35764
> 相关总表：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/common.html
> 适用版本：HBuilderX 5.13+

## 与 `uniapp.podspec` 的关系

官方 5.13+ `uniapp.podspec` 的 Pod subspec 对照表覆盖 DCloud 内置能力和模块；原生插件不是固定内置 subspec，不能直接按模块名写入 `uniapp_subspecs`。

```ruby
uniapp_subspecs = [
  'Core',
  # 内置模块写在这里；原生插件不要直接猜测 subspec 名称
]
```

## 插件自身的 Pod 依赖

原生插件如声明 CocoaPods 依赖，应按插件包内配置、插件市场说明或插件官方文档处理。官方原生插件文档要求根据插件 `package.json` 等配置映射到 Xcode 工程；通过 CocoaPods 引入的插件还需要确认：

- `Podfile` 中已正确声明 pod 依赖。
- 已执行 `pod install`。
- 使用 `.xcworkspace` 打开工程。
- CocoaPods 的 frameworks 搜索路径设置正确。

## 建议

- DCloud 内置模块优先使用本目录对应的 5.13+ `uniapp` subspec。
- 插件市场原生插件按插件自身文档处理，不要把插件名当作 `uniapp` subspec。
- UTS 插件优先参考 [uts-plugin-build.md](uts-plugin-build.md) 的 5.13+ 自动集成路径。
