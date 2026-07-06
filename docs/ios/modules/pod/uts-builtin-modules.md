# UTS 基础模块与内置模块 Pod 集成

> 官方文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/uts.html
> 官方源文档：https://gitee.com/dcloud/native-docs/blob/master/docs/AppDocs/usemodule/iOSModuleConfig/uts.md
> 适用版本：HBuilderX 5.13+

## Pod subspec

| 能力 | Pod subspec | 说明 |
| --- | --- | --- |
| UTS 基础模块 | `UTS` | UTS 运行支持 |

## Podfile 示例

```ruby
uniapp_subspecs = [
  'Core',
  'UTS',
]
```

## UTS 插件自动集成

官方 5.13+ 示例工程提供 `HBuilder-Hello/UTSPlugins` 目录。将 HBuilderX 导出资源中的 UTS 插件 iOS 目录复制到：

```text
HBuilder-Hello/UTSPlugins/<插件名称>/app-ios
```

然后在 `Podfile` 中启用 `UTS` 并执行：

```sh
pod install --no-repo-update
```

官方脚本会为插件生成本地 Pod，并处理插件 `app-ios` 下的源码、依赖库、资源、`Info.plist`、`UTS.entitlements` 和 `config.json` 中的部分配置。

## 仍需按官方文档配置

- 使用 UTS 插件、实人认证模块以及 UTS 内置模块前，需要先具备 UTS 基础能力。
- 官方 UTS 页面列出的内置模块如需在 UTS 插件中调用 uni API，仍需按文档确认 `DCloudUTSExtAPI.framework` 等依赖是否已处理。
- `config.json` 中三方依赖、权限、entitlements 等配置以官方 UTS 文档和插件自身声明为准。
