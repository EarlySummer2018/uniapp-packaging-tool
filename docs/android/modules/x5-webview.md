# Android X5 Webview（Android）

> **适用版本**：HBuilderX 5.0+
> **平台**：Android
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/androidModuleConfig/

## 11. Android X5 Webview

| 适用场景 | 路径 | 文件名 |
|---|---|---|
| 5+ APP | SDK/libs | `webview-x5-release.aar` |
| uni-app项目 | SDK/libs | `webview-x5-release.aar`, `weex_webview-x5-release.aar` |

> X5不需要单独添加配置，直接拷贝上述文件到libs下即可。

**Tips**：NDK配置时请去除x86、64位cpu的配置，建议仅配置"armeabi-v7a"，否则可能无法正常使用X5内核。

详细说明参考：[DCloud App集成 X5 内核（腾讯浏览服务TBS）说明](https://ask.dcloud.net.cn/article/36806)

---

### 相关模块

- [其他模块及国际化配置](other-modules.md) — 其他功能模块配置
- [FAQ](../faq.md) — FAQ第19条关于x5配置说明
