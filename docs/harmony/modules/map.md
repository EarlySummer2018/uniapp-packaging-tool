# 腾讯地图 (Map)

> **适用版本**：HBuilderX 5.0+ / uni-app 5.0+
> **平台**：HarmonyOS
> **官方文档**：https://nativesupport.dcloud.net.cn/AppDocs/usemodule/harmonyModuleConfig/map.html

---

## 概述

目前在鸿蒙端 uni-app 内置腾讯地图模块，使用 Web 端方案渲染地图。

## 模块依赖

无需额外配置模块依赖，腾讯地图为内置模块。

## 配置步骤

在 `entry/src/main/module.json5` 文件内配置腾讯地图 Key。

### 配置代码

```json
{
  "module": {
    // ... 其他配置
    "metadata": [
      {
        "name": "TENCENT_MAP_KEY",
        "value": "腾讯地图的key"
      }
    ],
    // ... 其他配置
  }
}
```

完整示例：

```json
{
  "module": {
    "name": "entry",
    "type": "entry",
    "metadata": [
      {
        "name": "TENCENT_MAP_KEY",
        "value": "你的腾讯地图Key"
      }
    ]
  }
}
```

## 注意事项

> 由于腾讯地图 key 在 Web 端方案内使用，因此在申请腾讯地图 key 时需要将域名白名单留空以便地图能正确加载出来。

### 申请 Key 要点

1. 访问[腾讯位置服务控制台](https://lbs.qq.com/console/mykey.html)申请 Key
2. 在"域名白名单"输入框中**留空**或设置为 `*`
3. 选择 Web端（JSAPI）类型的 Key

## 相关文档

- [← 返回模块总览](../index.md)
- [通用模块说明](./common.md)
