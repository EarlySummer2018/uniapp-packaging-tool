# UIWebview 配置（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/uiwebview.html

---

## Appstore 审核反馈废弃 UIWebview APIs 问题的说明

iOS 有 UIWebview 和 WKWebview 两种 webview。从 iOS 13 开始苹果将 UIWebview 列为过期 API。

**2020 年 4 月起 App Store 将不再接受使用 UIWebView 的新 App 上架、2020 年 12 月起将不再接受使用 UIWebView 的 App 更新。**

从 HBuilderX 2.2.5 起，iOS 上默认均已经是 WKWebview，除非开发者手动在代码中指定要用 UIWebview，否则实际渲染的页面都是在 WKWebview 里渲染的。不过，虽然实际页面是 WKWebview 渲染的，但 App 底层引擎源码里仍然有 UIWebview 的可选引用。Appstore 的机审会发现二进制代码中包括对 UIWebview 的引用，从而引发告警。从 HBuilderX 2.6.6 起，UIWebview 从基础引擎中移除，变成可选模块。

## iOS UIWebview 模块配置

如果开发者需要在离线打包工程中使用 UIWebview 功能，需要在自己的离线工程中配置 UIWebview 模块。

HBuilderX 5.13+ 如需集成 UIWebView 兼容模块，推荐使用本地 Pod，对应 Pod subspec 为 `UIWebview`。 手动集成时再参考下方依赖表。

### 添加依赖资源及文件

| 依赖库 | 系统库 | 依赖资源 |
|---|---|---|
| libH5WEUIWebview.a | JavaScriptCore.framework、Foundation.framework、UIKit.framework | 无 |

---

## 交叉引用

- 上一篇：[uni-AD（广告）](uni-ad.md)
- 下一篇：[UTS 内置模块](uts-builtin-modules.md)
- 相关模块：[UTS 内置模块](uts-builtin-modules.md)、[FAQ](../faq.md)
