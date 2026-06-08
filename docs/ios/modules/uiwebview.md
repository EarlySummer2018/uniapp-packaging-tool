# UIWebview 配置（iOS）

> **适用版本**：HBuilderX 5.0+
> **平台**：iOS (iPhone/iPad)
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/iOSModuleConfig/

---

> **重要提示**：从 iOS 12 开始，Apple 已弃用 UIWebview，推荐使用 WKWebview。
> 
> HBuilderX 3.0+ 版本默认使用 WKWebview，但如果项目中有特殊需求仍需使用 UIWebview，可参考以下配置。

## 需要引入的系统框架

| 框架 | 说明 |
|------|------|
| UIKit.framework | UI框架 |
| JavaScriptCore.framework | JavaScript 引擎 |

## Info.plist 配置

```xml
<!-- 允许任意加载（仅开发环境使用） -->
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>
```

## 需要拷贝的文件

| 路径 | 文件 |
|------|------|
| SDK/libs | `libUIWebview.a` 或相关 framework |

## Objective-C 代码示例

```objc
#import <UIKit/UIKit.h>

// 使用 UIWebView（不推荐，建议迁移至 WKWebView）
UIWebView *webView = [[UIWebView alloc] initWithFrame:self.view.bounds];
webView.delegate = self;

NSURL *url = [NSURL URLWithString:@"https://example.com"];
NSURLRequest *request = [NSURLRequest requestWithURL:url];
[webView loadRequest:request];

[self.view addSubview:webView];
```

## ⚠️ 迁移建议

1. **优先使用 WKWebView**：性能更好、内存占用更低、支持更多现代 Web 特性
2. **兼容性检查**：检查项目中是否有依赖 UIWebView 的第三方库
3. **App Store 审核**：2020年12月起，Apple 可能拒绝使用 UIWebView 的新应用
4. **迁移指南**：参考 Apple 官方的 [UIWebView Deprecation](https://developer.apple.com/documentation/uikit/uiwebview) 文档

---

## 交叉引用

- 上一篇：[uni-AD（广告）](uni-ad.md)
- 下一篇：[UTS 内置模块](uts-builtin-modules.md)
- 相关模块：[UTS 内置模块](uts-builtin-modules.md)、[FAQ - Q9 内存警告和应用崩溃](../faq.md#q9-内存警告和应用崩溃)
