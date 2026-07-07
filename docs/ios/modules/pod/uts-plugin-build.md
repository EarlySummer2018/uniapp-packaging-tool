# UTS 插件构建与 5.13+ Pod 集成

> 官方 UTS 文档：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/uts.html
> UTS 插件构建参考：https://doc.dcloud.net.cn/uni-app-x/native/use/iosuts.html
> 适用版本：HBuilderX 5.13+

## 5.13+ 推荐路径

对于 HBuilderX 5.13+ 示例工程，官方 UTS 模块页给出的推荐方式是：

1. 在 `Podfile` 的 `uniapp_subspecs` 中启用 `UTS`。
2. 将导出资源中的 `uni_modules/<插件名称>/app-ios` 复制到 `HBuilder-Hello/UTSPlugins/<插件名称>/app-ios`。
3. 通过示例工程 `scripts/uniapp_uts_plugins` 生成插件本地 Pod，并在 `target 'HBuilder'` 中追加插件 Pod。
4. 执行 `pod install --no-repo-update`。
5. 使用 `.xcworkspace` 打开工程。

```ruby
uniapp_subspecs = [
  'Core',
  'UTS',
]

uts_plugins = UniAppUTSPlugins.prepare!(
  File.join(__dir__, 'UTSPlugins'),
  sdk_path: File.expand_path('..', __dir__),
  values: defined?(UNIAPP_UTS_PLUGIN_VALUES) ? UNIAPP_UTS_PLUGIN_VALUES : {}
)

# 当前 CocoaPods 会校验本地插件 podspec 的 homepage、license、authors 等元数据；
# 如官方脚本生成的 podspec 缺少这些字段，需要在声明插件 Pod 前补齐。

target 'HBuilder' do
  pod 'uniapp', :path => '..', :subspecs => uniapp_subspecs
  uts_plugins.each do |plugin|
    pod plugin[:pod_name], :path => plugin[:pod_path]
  end
end
```

## 自动处理范围

官方说明中，脚本会为插件生成本地 Pod，并处理插件 `app-ios` 下的源码、依赖库、资源、`Info.plist`、`UTS.entitlements` 和 `config.json` 中的部分配置。

## 仍需按官方文档配置

- 插件的三方 Pod 依赖以插件 `config.json` 或插件官方说明为准。
- 如果插件使用特殊系统能力、证书、后台模式或需要手工合并的配置，仍需按插件文档处理。
- 旧版手动新建 framework、编译 xcframework 的流程可作为排查和兼容参考；5.13+ 优先使用官方本地 Pod 自动集成路径。
